function withOpacityValue(variable) {
    return ({opacityValue}) => {
        if (opacityValue === undefined) {
            return `hsla(var(${variable}),1)`;
        }
        return `hsla(var(${variable}) , ${opacityValue})`;
    };
}

module.exports = {
    content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
    theme: {
        extend: {
            colors: {
                primary: withOpacityValue("--color-primary"),
                secondary: withOpacityValue("--color-secondary"),
                tertiary: withOpacityValue("--color-tertiary"),
                quaternary: withOpacityValue("--color-quaternary"),
                accent: withOpacityValue("--color-accent"),
            },
        },
    },
    plugins: [],
};
