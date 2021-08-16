use handlebars::Path;

pub trait RepositoryResponse{
}

pub struct FileResponse{
    pub file: Path
}

impl  RepositoryResponse for FileResponse{

}

pub struct FileListingResponse{
    pub files: Vec<String>
}
impl RepositoryResponse for FileListingResponse{

}