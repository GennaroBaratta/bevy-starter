FROM mcr.microsoft.com/devcontainers/rust:2-1-trixie

RUN apt-get update \
    && apt-get install -y --no-install-recommends clang curl mold openjdk-21-jdk-headless unzip \
    && rm -rf /var/lib/apt/lists/*

ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_NDK_HOME=/opt/android-sdk/ndk/27.2.12479018
ENV JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
ENV PATH=/opt/android-sdk/cmdline-tools/latest/bin:/opt/android-sdk/platform-tools:${PATH}

COPY install-android-sdk.sh /tmp/install-android-sdk.sh
RUN bash /tmp/install-android-sdk.sh \
    && rm /tmp/install-android-sdk.sh \
    && chmod -R a+rX /opt/android-sdk
