const { Binary } = require("binary-install");
const os = require("os");
const cTable = require("console.table");

const error = (msg) => {
  console.error(msg);
  process.exit(1);
};

const { version, name } = require("./../package.json");

const supportedPlatforms = [
  {
    TYPE: "Windows_NT",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-pc-windows-msvc",
    BINARY_NAME: "lupinas-lullaby-win64.exe",
  },
  {
    TYPE: "Windows_NT",
    ARCHITECTURE: "x32",
    RUST_TARGET: "x86_64-pc-windows-msvc",
    BINARY_NAME: "lupinas-lullaby-win32",
  },
  {
    TYPE: "Linux",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-unknown-linux-musl",
    BINARY_NAME: "lupinas-lullaby-linux",
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-apple-darwin",
    BINARY_NAME: "lupinas-lullaby-macos",
  },
];

const getPlatformMetadata = () => {
  const type = os.type();
  const architecture = os.arch();

  for (let index in supportedPlatforms) {
    let supportedPlatform = supportedPlatforms[index];
    if (
      type === supportedPlatform.TYPE &&
      architecture === supportedPlatform.ARCHITECTURE
    ) {
      return supportedPlatform;
    }
  }

  error(
    `Platform with type "${type}" and architecture "${architecture}" is not supported by ${name}.\nYour system must be one of the following:\n\n${cTable.getTable(
      supportedPlatforms
    )}`
  );
};

const getBinary = () => {
  const platformMetadata = getPlatformMetadata();
  const url = `https://github.com/hbina/lupinas-lullaby/releases/download/v${version}/${platformMetadata.BINARY_NAME}.tar.gz`;
  return new Binary("lupinas-lullaby", url);
};

const run = () => {
  try {
    const binary = getBinary();
    binary.run(process.argv);
  } catch (e) {
    console.error(`${JSON.stringify(e)}`);
    process.exit(1);
  }
};

const install = () => {
  const binary = getBinary();
  binary.install();
};

module.exports = {
  install,
  run,
};
