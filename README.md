# Workshop: Einstieg in Embassy auf dem ESE Kongress 2025

Hier werden die Dateien für den Workshop "Einstieg in Embassy" hochgeladen.

---

## Installationsanleitung

Damit Sie direkt loslegen können, bereiten Sie bitte vorab die folgende Umgebung vor.

## Hardware

Sie erhalten im Workshop:

* **ESP32-C3-DevKit-RUST-1**

Das Entwicklungsboard hat einen USB-C Anschluss. Bitte bringen Sie ein USB-C Kabel mit, das an Ihr Notebook angeschlossen werden kann.

## Erforderliche Software

### 1. Rust Toolchain

Falls Rust noch nicht installiert ist:

* **Installation:** [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

Wir empfehlen die Installation über **rustup**. Dies installiert auch den Package-Manager **cargo**, den wir zur weiteren Installation verwerden werden.
Kurz vor dem Workshop bitte noch einmal ausführen:

```bash
rustup update
```

### 2. ESP32 Rust Toolchain

Fügen Sie die für den ESP32-C3 benötigten Komponenten hinzu:

```bash
# RISC-V Target für ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Rust-Src für std-Unterstützung
rustup component add rust-src
```

### 3. Espressif Tools

Installieren Sie die offiziellen Tools von Espressif:

```bash
# espup (Toolchain Manager)
cargo install espup
espup install

# Flashing-Tools
cargo install espflash cargo-espflash
```

Beachten Sie die Umgebungsvariablen, die nach `espup install` angezeigt werden.

### 4. Weitere empfohlene Tools

```bash
# Monitoring und Debugging
cargo install espmonitor

# Projekt-Templates für ESP32
cargo install cargo-generate
```

### Hinweise zu Betriebssystemen

* **Windows:** PowerShell empfohlen, einige Vorteile gegenüber der alten `cmd.exe`

---

## Troubleshooting vor Ort am Workshoptag

### USB-Treiber

**Windows/macOS:**
Der ESP32-C3 nutzt einen integrierten USB-Serial-JTAG-Controller. Moderne Systeme erkennen ihn normalerweise automatisch.
Falls das Board **am Tag des Workshops nicht erkannt wird**, können die Windows-Treiber hier gefunden werden:

* **Treiber:** [https://github.com/espressif/esp-win-usb-drivers](https://github.com/espressif/esp-win-usb-drivers)

**Linux:**
Fügen Sie Ihren Benutzer zur `dialout`-Gruppe hinzu (bzw. der `uucp`-Gruppe auf Arch Linux):

```bash
sudo usermod -a -G dialout $USER
# Danach ab- und anmelden
```

### Häufige Probleme

1. **"Device not found"**
   → USB-Kabel prüfen, anderen Port testen, ggf. Treiber installieren.

2. **Permission denied (Linux)**
   → Mitgliedschaft in der `dialout`-Gruppe prüfen. Ansonsten den Guide hier durchgehen: https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/get-started/establish-serial-connection.html

3. **rustup nach Installation nicht gefunden**
   → Terminal/Shell neu starten.
