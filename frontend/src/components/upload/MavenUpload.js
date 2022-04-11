import BasePlugin from '@uppy/core/lib/BasePlugin.js'

class MavenUploader extends BasePlugin {
    constructor(uppy, opts) {
        super(uppy, opts)
        this.id = opts.id || 'MavenUploader'
        this.title = "MavenUploader"
        this.type = 'maven'
        this.prepareUpload = this.prepareUpload.bind(this) // ← this!

    }
    prepareUpload (fileIDs) {
        console.log(this) // `this` refers to the `MyPlugin` instance.
        return Promise.resolve()
      }
    install() {
        this.uppy.addPreProcessor(this.prepareUpload)

    }

    uninstall() {
        this.uppy.removePreProcessor(this.prepareUpload)

    }
}