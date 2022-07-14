# The CI Artifact Repository

The goal of this repository is to store the artifacts of CI Builds. The motivation is that Github Actions zips the
artifacts. This will allow you to push the artifacts with archiving them

## Repository Layout

```
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
```

## CI Config

```json5
{
  // If empty, all projects are allowed
  "allowedProjects": [
    "nitro_repo",
    "adoptium-rs"
  ],
  // If empty, no retention policy is applied
  retentionPolicy: {
    numberOfBuilds: 10,
  },
}
```

## Authentication

The Authentication follows the same as the API and web interface.

1. Authentication Bearer [Token]
2. Authorization Basic [Username]:[Password]
3. Cookie for the web interface

### Pushing Build Artifacts

We follow a process similar to the one found in Maven

```
PUT /repository/storage/repository/{project}/{job_name}/{branch}/{build}/{artifact_name}
```

### Pushing a Directory

You can push a directory by compressing it into a ZIP or tar.gz

```
PUT /repository/storage/repository/{project}/{job_name}/{branch}/{build}?Directory=true
```

This will cause on PUT to decompress the directory. In the same layout within the archive

### Retrieving Artifacts

```
// Return a list of builds. Will just be a JSON array of build numbers
GET /repository/storage/repository/{project}/{job_name}/{branch}/
// Return a list of Artifacts. A JSON array of file names and their sizes. Also the time of publish
GET /repository/storage/repository/{project}/{job_name}/{branch}/{build}/
// Return the artifact. 
GET /repository/storage/repository/{project}/{job_name}/{branch}/{build}/{artifact_name}
```

### Deleting Artifacts

```
// Deletes a build
DELETE /repository/storage/repository/{project}/{job_name}/{branch}/{build}
// Delete the artifact. 
Delete /repository/storage/repository/{project}/{job_name}/{branch}/{build}/{artifact_name}
```

## Version ID

The VERSION ID is a string that looks like "{JobName}:{BranchName}:{BuildNumber}"

This is more used internally and for the web interface.