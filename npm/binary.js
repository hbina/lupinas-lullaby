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
    TAR_NAME: "lupinas-lullaby-win64",
    BINARY_NAME: "lupinas-lullaby-win64.exe",
  },
  {
    TYPE: "Linux",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-unknown-linux-musl",
    TAR_NAME: "lupinas-lullaby-linux",
    BINARY_NAME: "lupinas-lullaby-linux",
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-apple-darwin",
    TAR_NAME: "lupinas-lullaby-macos",
    BINARY_NAME: "lupinas-lullaby-macos",
  },
];

const getPlatformMetadata = () => {
  const type = os.type();
  const architecture = os.arch();
  for (const platform of supportedPlatforms) {
    if (type === platform.TYPE && architecture === platform.ARCHITECTURE) {
      return platform;
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
  const url = `https://github.com/hbina/lupinas-lullaby/releases/download/v${version}/${platformMetadata.TAR_NAME}.tar.gz`;
  return new Binary(platformMetadata.BINARY_NAME, url);
};

const run = () => {
  try {
    const binary = getBinary();
    binary.run(process.argv);
  } catch (e) {
    error(`${JSON.stringify(e)}`);
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
