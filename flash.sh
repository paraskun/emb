sudo mkdir -p /mnt/pico

sudo mount /dev/disk/by-id/usb-RPI_RP2_E0C9125B0D9B-0\:0-part1 /mnt/pico
sudo elf2uf2-rs -d $1
sudo umount /mnt/pico
