cargo build --release
agb-gbafix target/thumbv4t-none-eabi/release/ZenF -o export/ZenF.gba
agb-gbafix target/thumbv4t-none-eabi/release/ZenF -o export/web_gbajs2/resources/ZenF.gba
D:\Devel\gameboy\mGBA-0.10.3-win64\mGBA.exe export\ZenF.gba