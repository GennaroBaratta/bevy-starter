param([switch]$Debugger)

$ErrorActionPreference = "Stop"

Set-Location $PSScriptRoot

if ($Debugger) {
    $sdkRoot = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
    $adb = Join-Path $sdkRoot "platform-tools\adb.exe"
    $jdb = Join-Path $env:JAVA_HOME "bin\jdb.exe"
    $pwsh = (Get-Command pwsh -ErrorAction Stop).Source
    $watch = Start-Process $pwsh -ArgumentList @("-NoProfile", "-File", $PSCommandPath) -PassThru
    $lastAppPid = ""
    $forward = ""

    try {
        while (-not $watch.HasExited) {
            $appPid = [string](& $adb shell pidof com.example.bevystarter 2>$null)
            $appPid = $appPid.Trim()
            if (-not $appPid -or $appPid -eq $lastAppPid) {
                Start-Sleep -Milliseconds 500
                continue
            }

            $forward = [string](& $adb forward tcp:0 "jdwp:$appPid")
            $forward = $forward.Trim()
            if ($LASTEXITCODE -ne 0) {
                Start-Sleep -Milliseconds 500
                continue
            }

            Write-Host "Debugger attached to com.example.bevystarter ($appPid)"
            & $jdb -connect "com.sun.jdi.SocketAttach:hostname=localhost,port=$forward"
            $jdbExitCode = $LASTEXITCODE
            & $adb forward --remove "tcp:$forward" | Out-Null
            $forward = ""
            if ($jdbExitCode -eq 0) { $lastAppPid = $appPid }
        }
    }
    finally {
        if ($forward) { & $adb forward --remove "tcp:$forward" | Out-Null }
        if (-not $watch.HasExited) { Stop-Process -Id $watch.Id }
    }

    exit $watch.ExitCode
}

& cargo watch --no-process-group -w src -w assets -w Cargo.toml -w Cargo.lock -- android\gradlew.bat -p android launchDebug --console plain
exit $LASTEXITCODE
