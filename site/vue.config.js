const webpack = require("webpack");

let API_URL = process.env.API_URL;

if (!API_URL) {
  try {
    const config = require("./config.json");
    API_URL = config.API_URL;
  } catch (e) {
    if (e.name === "SyntaxError") {
      throw new Error("There is a syntax error in your config.json file.");
    }
    throw new Error("No config file or environment variables found. ");
  }
}

module.exports = {
  chainWebpack: (config) => {
    config.plugin("html").tap((args) => {
      args[0].title = "Nitro Repo";
      return args;
    });
  },
  configureWebpack: {
    plugins: [
      new webpack.DefinePlugin({
        API_URL: JSON.stringify(API_URL),
      }),
    ],
  },
};
