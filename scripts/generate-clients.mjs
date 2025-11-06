#!/usr/bin/env zx
import 'zx/globals';
import * as c from 'codama';
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import { getAllProgramIdls } from './utils.mjs';

// Instanciate Codama.
const [idl, ...additionalIdls] = getAllProgramIdls().map((idl) =>
  rootNodeFromAnchor(require(idl))
);
const codama = c.createFromRoot(idl, additionalIdls);

// Update programs.
codama.update(
  c.updateProgramsVisitor({
    yellowstoneShield: { name: 'shield' },
  })
);

// Update accounts.
codama.update(
  c.updateAccountsVisitor({
    policy: {
      size: 7,
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'shield'),
        c.constantPdaSeedNodeFromString('utf8', 'policy'),
        c.variablePdaSeedNode(
          'mint',
          c.publicKeyTypeNode(),
          'The mint of the token extension account'
        ),
      ],
    },
  })
);

codama.update(
  c.updateAccountsVisitor({
    policy_v2: {
      size: 39,
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'shield'),
        c.constantPdaSeedNodeFromString('utf8', 'policy'),
        c.variablePdaSeedNode(
          'mint',
          c.publicKeyTypeNode(),
          'The mint of the token extension account'
        ),
      ],
    },
  })
);

// Update instructions.
codama.update(
  c.updateInstructionsVisitor({
    createPolicy: {
      byteDeltas: [c.instructionByteDeltaNode(c.accountLinkNode('policy'))],
      accounts: {
        policy: { defaultValue: c.pdaValueNode('policy') },
        owner: { defaultValue: c.accountValueNode('payer') },
      },
    },
    addIdentity: {
      byteDeltas: [c.instructionByteDeltaNode(c.accountLinkNode('policy'))],
      accounts: {
        policy: { defaultValue: c.pdaValueNode('policy') },
        owner: { defaultValue: c.accountValueNode('payer') },
      },
    },
    closePolicy: {
      byteDeltas: [c.instructionByteDeltaNode(c.accountLinkNode('policy'))],
      accounts: {
        policy: { defaultValue: c.pdaValueNode('policy') },
        owner: { defaultValue: c.accountValueNode('payer') },
      },
    },
  })
);

// Set account discriminators.
const key = (name) => ({ field: 'kind', value: c.enumValueNode('Kind', name) });
codama.update(
  c.setAccountDiscriminatorFromFieldVisitor({
    policy: key('policy'),
  })
);

// Render JavaScript.
const jsClient = path.join(__dirname, '..', 'clients', 'js');
codama.accept(
  renderJavaScriptVisitor(path.join(jsClient, 'src', 'generated'), {
    prettierOptions: require(path.join(jsClient, '.prettierrc.json')),
  })
);

// Render Rust.
const rustClient = path.join(__dirname, '..', 'clients', 'rust');
codama.accept(
  renderRustVisitor(path.join(rustClient, 'src', 'generated'), {
    formatCode: true,
    crateFolder: rustClient,
  })
);

// Post-process Rust files to add enum casts
// The Rust renderer doesn't automatically cast enum values to u8,
// so we need to add the cast manually for discriminator constants
const policyRsPath = path.join(rustClient, 'src', 'generated', 'accounts', 'policy.rs');
let policyContent = await fs.readFile(policyRsPath, 'utf-8');
policyContent = policyContent.replace(
  /pub const POLICY_KIND: u8 = Kind::Policy;/g,
  'pub const POLICY_KIND: u8 = Kind::Policy as u8;'
);
await fs.writeFile(policyRsPath, policyContent);

// Post-process JavaScript files to fix @solana/kit v2 type names
// The JS renderer generates code for older @solana/kit versions,
// but v2 renamed types to have an "I" prefix (e.g., AccountMeta -> IAccountMeta)
const jsGeneratedPath = path.join(jsClient, 'src', 'generated');
const typeReplacements = [
  [/\bAccountMeta\b/g, 'IAccountMeta'],
  [/\bAccountSignerMeta\b/g, 'IAccountSignerMeta'],
  [/\bInstruction\b/g, 'IInstruction'],
  [/\bInstructionWithAccounts\b/g, 'IInstructionWithAccounts'],
  [/\bInstructionWithData\b/g, 'IInstructionWithData'],
];

async function fixTypeNames(filePath) {
  let content = await fs.readFile(filePath, 'utf-8');
  for (const [pattern, replacement] of typeReplacements) {
    content = content.replace(pattern, replacement);
  }
  await fs.writeFile(filePath, content);
}

// Fix type names in all instruction files
const instructionsPath = path.join(jsGeneratedPath, 'instructions');
const instructionFiles = await fs.readdir(instructionsPath);
for (const file of instructionFiles) {
  if (file.endsWith('.ts') && file !== 'index.ts') {
    await fixTypeNames(path.join(instructionsPath, file));
  }
}

// Fix type names in shared/index.ts
await fixTypeNames(path.join(jsGeneratedPath, 'shared', 'index.ts'));

// Remove unused isProgramDerivedAddress import and expectProgramDerivedAddress function
// These are generated but never used, causing bundler warnings
const sharedIndexPath = path.join(jsGeneratedPath, 'shared', 'index.ts');
let sharedContent = await fs.readFile(sharedIndexPath, 'utf-8');
// Remove isProgramDerivedAddress from the import list
sharedContent = sharedContent.replace(/\n  isProgramDerivedAddress,\n/,  '\n');
// Remove the expectProgramDerivedAddress function (including its JSDoc comment)
sharedContent = sharedContent.replace(
  /\/\*\*\n \* Asserts that the given value is a PDA\.\n \* @internal\n \*\/\nexport function expectProgramDerivedAddress<[^>]+>\(\n[\s\S]+?\n\}\n\n/,
  ''
);
await fs.writeFile(sharedIndexPath, sharedContent);