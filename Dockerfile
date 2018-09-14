FROM ubuntu:18.04
ARG DEBIAN_FRONTEND="noninteractive"
RUN dpkg --add-architecture i386 \
	&& apt-get update \
	&& apt-get install -y \
# Required for adding repositories
		software-properties-common \
# Required for wine
		winbind \
# Required for winetricks
		cabextract \
		p7zip \
		unzip \
		wget \
		curl \
		zenity \
# Install wine
	&& wget -O- https://dl.winehq.org/wine-builds/Release.key | apt-key add - \
	&& apt-add-repository https://dl.winehq.org/wine-builds/ubuntu/ \
	&& apt-get update \
	&& apt-get install -y --install-recommends winehq-devel \
# Download wine cache files
	&& mkdir -p /home/wine/.cache/wine \
	&& wget https://dl.winehq.org/wine/wine-mono/4.7.3/wine-mono-4.7.3.msi \
		-O /home/wine/.cache/wine/wine-mono-4.6.4.msi \
	&& wget https://dl.winehq.org/wine/wine-gecko/2.47/wine_gecko-2.47-x86.msi \
		-O /home/wine/.cache/wine/wine_gecko-2.47-x86.msi \
	&& wget https://dl.winehq.org/wine/wine-gecko/2.47/wine_gecko-2.47-x86_64.msi \
		-O /home/wine/.cache/wine/wine_gecko-2.47-x86_64.msi \
# Download winetricks and cache files
	&& wget https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks \
		-O /usr/bin/winetricks \
	&& chmod +rx /usr/bin/winetricks \
	&& mkdir -p /home/wine/.cache/winetricks/win7sp1 \
	&& wget https://download.microsoft.com/download/0/A/F/0AFB5316-3062-494A-AB78-7FB0D4461357/windows6.1-KB976932-X86.exe \
		-O /home/wine/.cache/winetricks/win7sp1/windows6.1-KB976932-X86.exe \
# Create user and take ownership of files
	&& groupadd -g 1010 wine \
	&& useradd -s /bin/bash -u 1010 -g 1010 wine \
	&& chown -R wine:wine /home/wine \
# Clean up
	&& apt-get autoremove -y \
		software-properties-common \
	&& apt-get autoclean \
	&& apt-get clean \
	&& apt-get autoremove
VOLUME /home/wine
ENV WINEARCH=win64
ENV WINEDEBUG=fixme-all
RUN winecfg

WORKDIR /app
COPY dxcompiler.dll /app
COPY dxil.dll /app
COPY dxil-signing.exe /app
COPY simple.dxil /app

ENTRYPOINT ["/bin/bash"]
#ENTRYPOINT ["wine", "dxil-signing.exe"]