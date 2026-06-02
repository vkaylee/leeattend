const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../webdashboard/src/i18n/locales');
const langs = ['vi', 'en', 'zh'];

function getFlatKeys(obj, prefix = '') {
  let keys = [];
  for (const [key, value] of Object.entries(obj)) {
    const newKey = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys = keys.concat(getFlatKeys(value, newKey));
    } else {
      keys.push(newKey);
    }
  }
  return keys;
}

try {
  // Get all namespace JSON files from the 'en' directory
  const enDir = path.join(localesDir, 'en');
  if (!fs.existsSync(enDir)) {
    console.error(`Error: 'en' locale directory not found at ${enDir}`);
    process.exit(1);
  }

  const jsonFiles = fs.readdirSync(enDir).filter(f => f.endsWith('.json'));

  let hasDiscrepancy = false;

  for (const jsonFile of jsonFiles) {
    console.log(`Checking namespace: ${jsonFile}...`);
    const fileKeys = {};

    for (const lang of langs) {
      const filePath = path.join(localesDir, lang, jsonFile);
      if (!fs.existsSync(filePath)) {
        console.error(`❌ File ${jsonFile} missing in ${lang}/`);
        hasDiscrepancy = true;
        continue;
      }

      const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
      fileKeys[lang] = new Set(getFlatKeys(data));
    }

    // Compare keys between languages
    for (let i = 0; i < langs.length; i++) {
      for (let j = i + 1; j < langs.length; j++) {
        const langA = langs[i];
        const langB = langs[j];
        const keysA = fileKeys[langA];
        const keysB = fileKeys[langB];

        if (!keysA || !keysB) continue;

        // Missing in B
        const missingInB = [...keysA].filter(x => !keysB.has(x));
        if (missingInB.length > 0) {
          console.error(`❌ Keys in ${jsonFile} present in ${langA}/ but missing in ${langB}/:`);
          missingInB.forEach(k => console.error(`  - ${k}`));
          hasDiscrepancy = true;
        }

        // Missing in A
        const missingInA = [...keysB].filter(x => !keysA.has(x));
        if (missingInA.length > 0) {
          console.error(`❌ Keys in ${jsonFile} present in ${langB}/ but missing in ${langA}/:`);
          missingInA.forEach(k => console.error(`  - ${k}`));
          hasDiscrepancy = true;
        }
      }
    }
  }

  if (hasDiscrepancy) {
    console.error('\n🔴 Localization key mismatch found! Translation keys must align 100% across namespaces.');
    process.exit(1);
  } else {
    console.log('✅ Localization keys reconciled successfully! All namespaces are aligned 100%.');
    process.exit(0);
  }
} catch (err) {
  console.error('Error running reconciliation check:', err);
  process.exit(1);
}
