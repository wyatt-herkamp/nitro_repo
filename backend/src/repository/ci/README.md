# The CI Artifact Repository

The goal of this repository is to store the artifacts of CI Builds. The motivation is that Github Actions zips the artifacts. This will allow you to push the artifacts with archiving them

## Repository Layout
Storage/
└── Repository (After this will be the different different projects)/
├── nitro_repo/
│   └── push(The Job Name)/
│       └── main(Branch Name. Optional)/
│           ├── 1/
│           │   ├── nitro_repo-linux.tar.gz
│           │   └── nitro_repo-windows.zip
│           ├── 2/
│           │   ├── nitro_repo-linux.tar.gz
│           │   └── nitro_repo-windows.zip
│           └── 3/
│               ├── nitro_repo-linux.tar.gz
│               └── nitro_repo-windows.zip
└── adoptium-rs/
└── push(The Job Name)/
├── 1/
│   └── adoptium
├── 2/
│   └── adoptium
└── 3/
└── adoptium