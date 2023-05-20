# Norgberg

Norgberg is the database services module for [Neorg](https://github.com/nvim-neorg/neorg), providing fast document data caching and module state storage. It is designed to follow the vision of the [Neorg Roadmap](https://github.com/nvim-neorg/neorg/blob/main/ROADMAP.md#external-tooling) to be a modular service that can be embedded in various different applications.

Norgberg serves two purposes.

1. It is a fast structured data cache for the information in and between notes of a neorg workspace, storing properties like metadata headers, link resolutions, tasks and other structured data in the neorg standard for fast querying by other modules such as Zettelkasten and Getting Things Done (GTD).
2. It is a state storage service that can be used by Neorg modules to persist information, for purposes such as synchronisation between devices, or holding module information.

Norgberg thus provides services found in other linked notes applications, as well as some utility beyond.

Norgberg is ment to be a modular services for modules plugging into the Neorg ecosystem - we explicitly want other modules to store their information in one common database service that is provided for all.

## Core design

The basis of Norgberg is an encapsulation of an SQLite database with custom schemas, exposed through an API. It stores primary key references for all unique files in a Neorg workspace. This allows the SQLIte database to store link resolutions between files and other such information. The database is populated by consuming the Tree-sitter Concrete Syntax Trees provided by Neorg's multi-threaded parses, and held up to date by updating database state when Neorg buffers are written to file. The API allows users to run fast queries on the DB to resolve links for jumps, aggregate and filter tasks across a workspace, and other queries of interest.

In the beginning, Norgberg will only store and maintain select information caching. In the long-term, we hope to automatically cache most to all of the structured metadata in the Norg specifications by default, allowing users easy and fast access to the structured information in their workspaces with minimal custom effort.

In addition, other modules may add new tables to the SQLite database to store their own information. This capability should be used with the understanding that any module state stored in the database this way will not be stored by Norgberg in any other files, and may be lost in case of database corruption or deletion. Norgberg itself will only reconstruct state that exists in the Neorg files.

Long-term, we aim to introduce the caching of neorg files data in a graph database, allowing for graph queries and analysis with high performance and purposeful query languges. This should empower advanced applications using attached modifiers, tag extensions, and looking at many-notes data structures. Initial graph services will be provided through SQLite schemas for representing edge connections between nodes.

### Interface

Norgberg runs as a module of the [Norgopolis common backend server](https://github.com/SevorisDoe/Norgberg). It conforms to the Norgopolis RPC standard. Methods exposed by Norgberg can be called via the common RPC router from other modules or via gRPC frontends in your application or script, as long as the method name is available and the mpack deserializes to matche the method arguments.

### Cookie system

The cookie system enforces database tidieness and allows modules to automatically migrate database schemas when upgrades occur. The system stores information about what other modules have registered themselves with the database using the modules name and semantic version. On first communication after startup, modules communicate their current semantic version. Norgberg can inform the modules if version changes occured, which can be used to trigger automatic migration logic.

The cookie system also tracks which tables are claimed by what module. This information can be used by the user to clean up unwanted tables cluttering up the database.

## Roadmap

Currently, we are aiming to provide the core first-generation SQLite database to support the GTD and Zettelkasten modules with key cache and storage services related to inter-file links and tasks.

- [ ] Design the interface API
- [ ] Design the database populating code
- [ ] Design the state-updating code
- [ ] Design startup, shutdown and many-modules connectivity

We are also aiming to develop a benchmark for graph-like information manipulation on Norgberg, helping us quantify the performance of the SQLIte graph-modelling schemas and how they compare to purposeful graph-paradigm databases we are planning to introduce long-term. For this benchmark we are looking for generalized descriptions of note-worthy link structures people have in their vaults. The more examples we have, the better!

For graph databases, we are currently evaluating multiple candidates
- SurrealDB
- Oxigraph
- CozoDB
  
With special interest in CozoDB for its support of SQLite as a storage engine, use of the Datalog query language, and support of relational, graph, and vector paradigma.

## Contributing
Especially important in this early phase of Norgberg's existence is clarity about what information needs storing and retrieving and along what schema! This is a call to everyone else looking to roll out modules for Neorg.

We also need to decide on a first way in which multiple different Rust modules are linked together into one system on the user's system, in a way that is both performant at runtime and not too much effort on the user's part.

Beyond that, all thoughts and experience is welcome! In the coming days, this repo will be updated with a growing set of design deliberations and decisions.
