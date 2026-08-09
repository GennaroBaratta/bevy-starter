#!/usr/bin/env -S node --experimental-strip-types

import { accessSync, constants } from "node:fs";
import { homedir } from "node:os";
import { delimiter, join } from "node:path";
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import process from "node:process";

const APP_ID = "com.example.bevystarter";
const root = import.meta.dirname;
const debuggerEnabled = process.argv.slice(2).some((argument) =>
  ["--debugger", "-debugger"].includes(argument.toLowerCase()),
);

process.chdir(root);

function findAndroidSdk(): string {
  const adbName = process.platform === "win32" ? "adb.exe" : "adb";
  const candidates = [
    process.env.ANDROID_HOME,
    process.env.ANDROID_SDK_ROOT,
    process.env.LOCALAPPDATA && join(process.env.LOCALAPPDATA, "Android", "Sdk"),
    join(homedir(), "Android", "Sdk"),
    join(homedir(), ".android-sdk"),
    "/opt/android-sdk",
  ];

  for (const candidate of candidates) {
    if (!candidate) continue;
    try {
      accessSync(join(candidate, "platform-tools", adbName), constants.X_OK);
      return candidate;
    } catch {
      // Try the next conventional SDK location.
    }
  }

  throw new Error("Could not find the Android SDK. Set ANDROID_HOME or ANDROID_SDK_ROOT.");
}

const sdkRoot = findAndroidSdk();
const childEnvironment = {
  ...process.env,
  ANDROID_HOME: sdkRoot,
  ANDROID_SDK_ROOT: sdkRoot,
};

const adbName = process.platform === "win32" ? "adb.exe" : "adb";
const adb = executable([join(sdkRoot, "platform-tools", adbName)], adbName);

function executable(candidates: Array<string | undefined>, name: string): string {
  for (const candidate of candidates) {
    if (!candidate) continue;
    try {
      accessSync(candidate, constants.X_OK);
      return candidate;
    } catch {
      // Try the next explicit path or PATH entry.
    }
  }

  for (const directory of (process.env.PATH ?? "").split(delimiter)) {
    const candidate = join(directory, name);
    try {
      accessSync(candidate, constants.X_OK);
      return candidate;
    } catch {
      // Keep searching PATH.
    }
  }

  throw new Error(`Could not find ${name}. Check your dev-container environment.`);
}

function run(command: string, args: string[], stdio: "inherit" | "pipe" = "inherit") {
  return spawnSync(command, args, { encoding: "utf8", env: childEnvironment, stdio });
}

function requireAndroidDevice(): void {
  const result = run(adb, ["devices"], "pipe");
  if (result.status !== 0) {
    const details = result.stderr.trim();
    throw new Error(
      `Could not query ADB.${details ? `\n${details}` : ""}`,
    );
  }

  const devices = result.stdout
    .split(/\r?\n/)
    .slice(1)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [serial, state] = line.split(/\s+/, 2);
      return { serial, state };
    });

  if (devices.some(({ state }) => state === "device")) return;

  const unauthorized = devices.filter(({ state }) => state === "unauthorized");
  if (unauthorized.length > 0) {
    throw new Error(
      `Android device ${unauthorized.map(({ serial }) => serial).join(", ")} is not authorized. ` +
      "Unlock it and accept the USB debugging prompt, then run this command again.",
    );
  }

  const offline = devices.filter(({ state }) => state === "offline");
  if (offline.length > 0) {
    throw new Error(
      `Android device ${offline.map(({ serial }) => serial).join(", ")} is offline. ` +
      "Reconnect it (or restart ADB), then run this command again.",
    );
  }

  throw new Error(
    "No Android device is connected. Connect a device with USB debugging enabled or start an emulator, " +
    "confirm it appears in `adb devices`, then run this command again.",
  );
}

function startWatcher(): ChildProcess {
  return spawn(
    "cargo",
    [
      "watch",
      "--no-process-group",
      "-w", "src",
      "-w", "assets",
      "-w", "Cargo.toml",
      "-w", "Cargo.lock",
      "--shell",
      "sh ./android/gradlew -p android launchDebug --console plain",
    ],
    { cwd: root, env: childEnvironment, stdio: "inherit" },
  );
}

function wait(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function watchWithDebugger(): Promise<number> {
  const jdb = executable([process.env.JAVA_HOME && join(process.env.JAVA_HOME, "bin", "jdb")], "jdb");
  const watcher = startWatcher();
  let forwardedPort = "";
  let lastAppPid = "";
  let stopping = false;

  const stop = () => {
    if (stopping) return;
    stopping = true;
    if (forwardedPort) run(adb, ["forward", "--remove", `tcp:${forwardedPort}`], "pipe");
    watcher.kill("SIGTERM");
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);

  try {
    while (watcher.exitCode === null && !stopping) {
      const pidResult = run(adb, ["shell", "pidof", APP_ID], "pipe");
      const appPid = pidResult.status === 0 ? pidResult.stdout.trim() : "";
      if (!appPid || appPid === lastAppPid) {
        await wait(500);
        continue;
      }

      const forwardResult = run(adb, ["forward", "tcp:0", `jdwp:${appPid}`], "pipe");
      forwardedPort = forwardResult.status === 0 ? forwardResult.stdout.trim() : "";
      if (!forwardedPort) {
        await wait(500);
        continue;
      }

      console.log(`Debugger attached to ${APP_ID} (${appPid})`);
      const jdbResult = run(jdb, ["-connect", `com.sun.jdi.SocketAttach:hostname=localhost,port=${forwardedPort}`]);
      run(adb, ["forward", "--remove", `tcp:${forwardedPort}`], "pipe");
      forwardedPort = "";
      if (jdbResult.status === 0) lastAppPid = appPid;
    }
  } finally {
    stop();
  }

  return await new Promise((resolve) => {
    if (watcher.exitCode !== null) resolve(watcher.exitCode);
    else watcher.once("exit", (code) => resolve(code ?? 1));
  });
}

async function main(): Promise<number> {
  requireAndroidDevice();

  if (debuggerEnabled) return watchWithDebugger();

  const watcher = startWatcher();
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.once(signal, () => watcher.kill(signal));
  }
  return await new Promise((resolve) => watcher.once("exit", (code) => resolve(code ?? 1)));
}

main()
  .then((exitCode) => process.exit(exitCode))
  .catch((error: unknown) => {
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  });
