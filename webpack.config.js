const path = require('path');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const webpack = require('webpack');

module.exports = (env, argv) => {
    const isProduction = argv.mode == 'production';

    return {
        plugins: [new MiniCssExtractPlugin()],
        entry: {
            bundle: './ui/index.js',
            style: './ui/index.scss'
        },
        output: {
            filename: '[name].js',
            path: path.resolve(__dirname, 'resources', 'app'),
            publicPath: ""
        },
        resolve: {
            alias: {
                Utilities: path.resolve('ui/js/')
            },
            extensions: ['.js', '.json', '.jsx']
        },
        devtool: (isProduction) ? "none" : "source-map",
        module: {
            rules: [
                {
                    test: /\.jsx?$/,
                    exclude: /node_modules/,
                    use: {
                        loader: 'babel-loader',
                        options: {
                            presets: [
                                '@babel/preset-env',
                                [
                                    '@babel/preset-react', {
                                        development: !isProduction
                                    }
                                ]
                            ]
                        }
                    }
                },
                {
                    test: /\.s[ac]ss$/i,
                    use: [
                        MiniCssExtractPlugin.loader,
                        {
                            loader: "css-loader",
                            options: {
                                // always make sourceMap. resolver-url-loader is needing it
                                "sourceMap": !isProduction,
                            },
                        },
                        "resolve-url-loader",
                        {
                            loader: "sass-loader",
                            options: {
                                sourceMap: true,
                            }
                        }
                    ] 
                    /*
                    use: [
                        MiniCssExtractPlugin.loader,
                        {
                            loader: "css-loader",
                            options: {
                                "sourceMap": !isProduction,
                            }
                        },
                        "resolve-url-loader",
                        {
                            loader: "sass-loader",
                            options: {
                                // always make sourceMap. resolver-url-loader is needing it
                                "sourceMap": true,
                            }
                        }
                    ]
                    */
                },
            ]
        }
    }
}
