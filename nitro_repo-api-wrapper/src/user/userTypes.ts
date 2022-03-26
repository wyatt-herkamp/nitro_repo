

export interface User {
    id: number;
    name: string;
    username: string;
    email: string;
    permissions: UserPermissions;
    created: number;
}

export interface UserListResponse {
    id: number;
    name: string;
}

export interface UserPermissions {
    admin: boolean;
    deployer: boolean;
}


export interface UserList {
    users: Array<User>;
}


export interface AuthToken {
    id: number;
    user: number;
    token: string;
    expiration: number;
    created: number;
}
