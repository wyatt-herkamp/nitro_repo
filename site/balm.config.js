const api = (mix) => {
    if (mix.env.isDev) {
        mix.copy('node_modules/balm-ui/fonts/*', 'app/fonts');
    }
};