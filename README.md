# MiniRDBMS

**MiniRDBMS - A Relational Database Engine in Rust**

MiniRDBMS is a lightweight relational database management system built from scratch in Rust to explore the internal components behind a database engine.

This project is an educational implementation. It is useful for studying storage, buffering, records, relations, and a SQL-like command loop, but it is not production-ready.

## Overview

MiniRDBMS provides an interactive CLI for creating databases and tables, inserting records, bulk-loading CSV-style data, and running single-table `SELECT` queries with optional filtering and projection.

The implementation keeps the architecture intentionally small:

```text
CLI
 |
 v
SGBD command dispatcher
 |
 v
DBManager ---- Database ---- Relation/Table
 |                              |
 |                              v
 |                         Records and scans
 v
BufferManager <----------> DiskManager
 |                              |
 v                              v
Buffer pool              Page files in dbpath/BinData
```

## Architecture

- `SGBD`: interactive command loop and command dispatch.
- `DBManager`: database catalog, current database selection, table registration, and metadata persistence.
- `Database`: in-memory collection of relations.
- `Relation`: table schema, header page, data pages, record insertion, record reading, and full scans.
- `DiskManager`: page allocation, page deallocation, page reads/writes, and persisted free-page state.
- `BufferManager`: fixed-size buffer pool with pin counts, dirty bits, and LRU/MRU replacement methods.
- `Select`, `Condition`, and operators: parsing and execution for single-table selection, filtering, projection, relation scanning, and record printing.

## Features

- Page-based storage using fixed-size pages.
- Disk-backed data files under the configured `dbpath`.
- Buffer pool with dirty-page flushing and configurable replacement policy (`LRU` or `MRU`).
- Database creation, listing, selection, and dropping.
- Table creation, listing, and dropping.
- Record insertion into relations.
- Bulk insertion from comma-separated files.
- Metadata persistence for databases and table schemas.
- Single-table `SELECT` parsing with aliases.
- Relation scanning, filtering with comparison operators, and column projection.

## Supported Commands

Commands are entered in the interactive CLI. The dispatcher expects uppercase command keywords.

```sql
CREATE DATABASE database_name
SET DATABASE database_name
LIST DATABASES
DROP DATABASE database_name
DROP DATABASES

CREATE TABLE table_name (column_name:TYPE,column_name:TYPE)
LIST TABLES
DROP TABLE table_name
DROP TABLES

INSERT INTO table_name VALUES (value,value,value)
BULKINSERT INTO table_name file.csv

SELECT * FROM table_name alias
SELECT alias.column_name,alias.column_name FROM table_name alias WHERE alias.column_name > 10
QUIT
```

Supported column types in the storage layer are:

- `INT`
- `REAL`
- `CHAR(n)`
- `VARCHAR(n)`

Supported `WHERE` comparison operators are:

- `=`
- `<>`
- `<`
- `>`
- `<=`
- `>=`

`WHERE` operands may compare alias-qualified columns, numeric constants, or quoted string constants. Multiple conditions may be separated with `AND`.

## Example Usage

```sql
CREATE DATABASE my_database
SET DATABASE my_database
LIST DATABASES

CREATE TABLE users (id:INT,name:VARCHAR(32))
LIST TABLES

INSERT INTO users VALUES (1,"Alice")
BULKINSERT INTO users data.csv

SELECT * FROM users u
SELECT u.id,u.name FROM users u WHERE u.id > 10

QUIT
```

## Getting Started

### Requirements

- Rust and Cargo, installed with [rustup](https://www.rust-lang.org/tools/install).

### Installation

```bash
git clone https://github.com/JuriSOK/MiniSGBDR.git
cd MiniSGBDR
cargo check
```

### Running the CLI

```bash
cargo run
```

The default `config.json` stores development data under `res/dbpath`.

## Configuration

`config.json` is loaded at startup and currently expects string values:

```json
{
  "dbpath": "res/dbpath",
  "pagesize": "4096",
  "dm_maxfilesize": "65536",
  "bm_buffer_count": "4",
  "bm_policy": "LRU"
}
```

- `dbpath`: directory used for metadata and page files.
- `pagesize`: fixed page size in bytes.
- `dm_maxfilesize`: maximum size of one disk-manager data file in bytes.
- `bm_buffer_count`: number of buffers in the buffer pool.
- `bm_policy`: replacement policy, either `LRU` or `MRU`.

## Project Structure

```text
.
|-- Cargo.toml
|-- config.json
|-- README.md
`-- src
    |-- buffer.rs
    |-- buffer_manager.rs
    |-- col_info.rs
    |-- condition.rs
    |-- config.rs
    |-- data_base.rs
    |-- db_manager.rs
    |-- disk_manager.rs
    |-- main.rs
    |-- operator.rs
    |-- page.rs
    |-- page_info.rs
    |-- record.rs
    |-- record_id.rs
    |-- relation.rs
    |-- select.rs
    |-- sgbd.rs
    `-- types.rs
```

## Technical Concepts Explored

- Slotted-page style record placement.
- Header pages and data pages.
- Disk page allocation and free-page tracking.
- Buffer pool management with pin counts and dirty bits.
- LRU/MRU page replacement strategies.
- Catalog persistence for databases and relations.
- SQL-like command parsing.
- Iterator-style relational operators for scan, selection, projection, and printing.

## Limitations

- This is an educational database engine, not a production system.
- `SELECT` execution is limited to one table.
- `FROM` entries must include an alias, even for single-table queries.
- Joins, indexes, transactions, concurrency control, recovery, and SQL DDL/DML completeness are not implemented.
- The parser is intentionally narrow and expects command syntax close to the examples above.
- Input validation and error recovery are limited; malformed commands may still panic in some paths.
- Values are handled as strings at the CLI boundary and converted by the storage/condition code where needed.

## Contributors

- [SOK VIBOL ARNAUD](https://github.com/JuriSOK)
- [MOUSTACHE MATHIEU](https://github.com/whoismathieu)
- [LETACONNOUX AYMERIC](https://github.com/Shrek1515)
- [MEUNIER YOHANN](https://github.com/Ora-197)

## Project Status

MiniRDBMS is a portfolio and learning project. The current focus is preserving the existing educational implementation while presenting it clearly for an English-language developer audience.
