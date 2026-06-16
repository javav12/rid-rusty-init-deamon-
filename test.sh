
RUSTFLAGS="-C target-feature=+crt-static" \
      cargo build --release \
      --target x86_64-unknown-linux-musl

cp target/x86_64-unknown-linux-musl/release/rid ../imgs/init
  chmod +x ../imgs/init

cd ../imgs

find . -print0 | cpio --null -ov --format=newc > ../initramfs.cpio

cd ../

qemu-system-x86_64 \
         -enable-kvm \
         -m 1G \
         -kernel ./bzImage \
         -initrd initramfs.cpio \
         -append "console=ttyS0 init=/init" \
         -nographic \
         -serial mon:stdio \
         -net nic,model=virtio \
         -net user
