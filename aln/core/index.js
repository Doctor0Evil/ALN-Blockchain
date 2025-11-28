/**
 * ALN Core - Main Entry Point
 */

const StateStore = require('./state/state_store');
const SoloConsensus = require('./consensus/solo_consensus');
const HttpServer = require('./api/http_server');
const { parseAlnDocument, validateChainlexemes } = require('./runtime/aln_parser');
const { BlockHeader, Block, applyBlock } = require('./ledger/block_model');
const { loadProfile, loadProfileWithDefaults } = require('./config/nanotopologene_loader');
const { Logger, ErrorCodes } = require('./logging/logger');
const { verifyConservation, verifyLimits, verifySafety } = require('./safety/qpu_math_hooks');

module.exports = {
  // State management
  StateStore,

  // Consensus
  SoloConsensus,

  // API
  HttpServer,

  // Parsing and validation
  parseAlnDocument,
  validateChainlexemes,

  // Ledger
  BlockHeader,
  Block,
  applyBlock,

  // Configuration
  loadProfile,
  loadProfileWithDefaults,

  // Logging
  Logger,
  ErrorCodes,

  // Safety
  verifyConservation,
  verifyLimits,
  verifySafety
};
