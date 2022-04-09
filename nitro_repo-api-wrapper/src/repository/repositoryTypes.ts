
export interface Repository {
    id: number;
    name: string;
    repo_type: string;
    settings: RepoSettings;
    deploy_settings: DeploySettings;
    security: SecurityRules;
    storage: string;
    created: number;
}
export interface ReportGeneration {
    active: boolean;
    values: Array<string>;
}

export interface Webhook {
    id: string;
    handler: string;
    settings: Map<String, any>;
}
export interface RepositoryListResponse {
    name: string;
    repo_type: string;
    storage: string;
}
export interface RepositoryList {
    repositories: Array<RepositoryListResponse>;
}
export interface SecurityRules {
    visibility: string;
    readers: Array<number>;
    deployers: Array<number>;
}
export interface BadgeSettings {
    style: string;
    label_color: string;
    color: string;
}export interface RepoSettings {
    policy: string;
    active: boolean;
    re_deployment: boolean;
    frontend: Frontend;
    badge: BadgeSettings;
}
export interface Frontend {
    page_provider: string;
    enabled: boolean;
}

export interface DeploySettings {
    report_generation: ReportGeneration;
    webhooks: Array<Webhook>;
}

export interface Version {
    version: string;
    artifacts: string[];
}

export interface RepoSummary {
    name: string;
    storage: string;
    page_provider: string;
    repo_type: string;
    visibility: string;
}

export interface Project {
    repo_summary: RepoSummary;
    project: ProjectData;
    frontend_response: null;
}

export interface ProjectData {
    name: string;
    description: string;
    source: null;
    licence: null;
    versions: Versions;
    created: number;
}

export interface Versions {
    latest_version: string;
    latest_release: string;
    versions: Version[];
}

export interface Version {
    version: string;
    time: number;
    snapshot: boolean;
}

export enum Policy {
    Release,
    Snapshot,
    Mixed,
}

export interface VersionBrowseResponse {
    Project?: ProjectData
    Version: string
}
export interface ResponseType {
    Project?: ProjectData
    Repository?: Repository
    Version?: VersionBrowseResponse
}
export interface BrowseResponse {
    response_type: ResponseType | string;
    files: Array<FileResponse>
}
export interface FileResponse {
    name: string;
    full_path: string;
    directory: boolean;
    file_size: number;
    response_type: ResponseType | string;
    created: number;
}
