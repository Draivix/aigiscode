#!/usr/bin/env node
import fs from 'fs/promises';
import { createRequire } from 'module';
import path from 'path';
import process from 'process';

const require = createRequire(import.meta.url);
const kuzuModulePath = process.env.AIGISCODE_NODE_MODULES
  ? path.join(process.env.AIGISCODE_NODE_MODULES, 'kuzu')
  : 'kuzu';
const kuzu = require(kuzuModulePath);

const NODE_TABLE = 'CodeNode';
const REL_TABLE = 'CodeRelation';

const NODE_SCHEMA = `
CREATE NODE TABLE ${NODE_TABLE}(
  id STRING,
  kind STRING,
  name STRING,
  qualifiedName STRING,
  filePath STRING,
  language STRING,
  parentSymbolId STRING,
  ownerTypeName STRING,
  returnTypeName STRING,
  visibility STRING,
  parameterCount INT64,
  requiredParameterCount INT64,
  startLine INT64,
  endLine INT64,
  PRIMARY KEY(id)
)`;

const REL_SCHEMA = `
CREATE REL TABLE ${REL_TABLE}(
  FROM ${NODE_TABLE} TO ${NODE_TABLE},
  type STRING,
  referenceKind STRING,
  relationKind STRING,
  layer STRING,
  strength STRING,
  origin STRING,
  resolutionTier STRING,
  confidenceMillis INT64,
  reason STRING,
  line INT64,
  occurrenceCount INT64
)`;

async function cleanupDb(dbPath) {
  try {
    const stat = await fs.stat(dbPath);
    if (stat.isDirectory()) {
      await fs.rm(dbPath, { recursive: true, force: true });
    } else {
      await fs.unlink(dbPath);
    }
  } catch {}

  for (const suffix of ['.wal', '.lock']) {
    try {
      await fs.unlink(`${dbPath}${suffix}`);
    } catch {}
  }
}

function normalizePath(filePath) {
  return filePath.replace(/\\/g, '/');
}

async function openDb(dbPath) {
  await fs.mkdir(path.dirname(dbPath), { recursive: true });
  const db = new kuzu.Database(dbPath);
  const conn = new kuzu.Connection(db);
  return { db, conn };
}

async function materialize(dbPath, nodeCsv, relCsv) {
  await cleanupDb(dbPath);
  const { db, conn } = await openDb(dbPath);
  try {
    await conn.query(NODE_SCHEMA);
    await conn.query(REL_SCHEMA);

    const nodeCopy = `COPY ${NODE_TABLE} FROM "${normalizePath(nodeCsv)}" (HEADER=true, ESCAPE='"', DELIM=',', QUOTE='"', PARALLEL=false, auto_detect=false)`;
    const relCopy = `COPY ${REL_TABLE} FROM "${normalizePath(relCsv)}" (from="${NODE_TABLE}", to="${NODE_TABLE}", HEADER=true, ESCAPE='"', DELIM=',', QUOTE='"', PARALLEL=false, auto_detect=false)`;
    await conn.query(nodeCopy);
    await conn.query(relCopy);
    process.stdout.write(JSON.stringify({ dbPath }));
  } finally {
    await conn.close?.();
    await db.close?.();
  }
}

async function query(dbPath, cypher) {
  const { db, conn } = await openDb(dbPath);
  try {
    const result = await conn.query(cypher);
    const rows = await result.getAll();
    let columns = [];
    if (rows.length > 0) {
      columns = Object.keys(rows[0]);
    }
    process.stdout.write(
      JSON.stringify({
        columns,
        rows,
        row_count: rows.length,
      }),
    );
  } finally {
    await conn.close?.();
    await db.close?.();
  }
}

async function main() {
  const [, , command, ...rest] = process.argv;
  if (command === 'materialize') {
    if (rest.length !== 3) {
      throw new Error('usage: kuzu_bridge.mjs materialize <db-path> <nodes.csv> <relations.csv>');
    }
    await materialize(rest[0], rest[1], rest[2]);
    return;
  }
  if (command === 'query') {
    if (rest.length !== 2) {
      throw new Error('usage: kuzu_bridge.mjs query <db-path> <cypher>');
    }
    await query(rest[0], rest[1]);
    return;
  }
  throw new Error(`unsupported command: ${command ?? '<missing>'}`);
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`${message}\n`);
  process.exit(1);
});
