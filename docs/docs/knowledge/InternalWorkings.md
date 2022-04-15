# Internal Workings
## Overview 
In general Nitro_Repo adds extra processing steps to the deploy process and artifact management. 
This is so that Nitro_Repo can provide quicker and easier access to information. It will also allow for a standardized way of knowing information about different artifacts. 

## Post Deploy Tasks
After deploying to Nitro_Repo. NR will take what it just got and process what it got. For Example. It will process the Pom.XML from a Maven deploy and get information such as name, LICENSE, and description and create a new file called `project.nitro_repo`. This is so now we can easily process this information back out. So if Mavens standard did change. We would only need to change how we took the data. All past deploys we still have our information and standard. We also collect additional information such as the time. 

## Storages
Before you create a Repository you create a `storage`. A storage is basically a location where the repositories will be listed. As of now all(1.0.0) this is hardset and cannot be changed. However, in the future. We will add the ability to do different locations or even different servers.

## Repositories
After creating a Storage. You create a repository. The repository belongs to the storage. And all its data is kept within the storage. 

## Users
Users are stored within the Mysql Database. In the future will change this to allow different storage methods of users. But as of now a MySQL database is required for the running of Nitro_Repo. We also create a `session_tokens` table. This is just the session tokens used by the website frontend for calling the backend API. Again in the future this will change and be more dynamic. If you are interested how and when. Please follow this [Issue](https://github.com/wherkamp/nitro_repo/issues/326).

