{
  rustPlatform,
  lib,
}:
rustPlatform.buildRustPackage {
  pname = "lucifer";
  version = "0.1.0";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  buildInputs = [
  ];

  meta = {
    description = "A DNS proxy and blocker";
    homepage = "https://github.com/myume/lucifer";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [myume];
    platforms = lib.platforms.all;
  };
}
