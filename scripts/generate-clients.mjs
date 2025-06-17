#!/usr/bin/env zx
import 'zx/globals';
import * as c from 'codama';
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import { renderVisitor as renderParserVisitor } from '@codama/renderers-vixen-parser';
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
