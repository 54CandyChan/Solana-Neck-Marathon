import fs from "fs";
import path from "path";

const newProgramId = process.argv[2];

if (!newProgramId) {
  console.error("Usage: node scripts/set-program-id.mjs <PROGRAM_ID>");
  process.exit(1);
}

const projectRoot = process.cwd();
const anchorTomlPath = path.join(projectRoot, "Anchor.toml");
const libRsPath = path.join(projectRoot, "programs", "cgd_store", "src", "lib.rs");

const replaceInFile = (filePath, replacer) => {
  const original = fs.readFileSync(filePath, "utf8");
  const updated = replacer(original);

  if (updated === original) {
    console.error(`No changes made in ${filePath}. Check the file format.`);
    process.exit(1);
  }

  fs.writeFileSync(filePath, updated, "utf8");
};

replaceInFile(anchorTomlPath, (content) =>
  content.replace(
    /cgd_store = "[^"]+"/,
    `cgd_store = "${newProgramId}"`
  )
);

replaceInFile(libRsPath, (content) =>
  content.replace(
    /declare_id!\("[^"]+"\);/,
    `declare_id!("${newProgramId}");`
  )
);

console.log(`Updated Program ID to ${newProgramId}`);
