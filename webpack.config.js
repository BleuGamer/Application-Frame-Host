const path = require('path');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const webpack = require('webpack');

module.exports = (env, argv) => {
    const isProduction = argv.mode == 'production';

    return {
        plugins: [
            new MiniCssExtractPlugin({
                filename: "[name].css"
            }),
            
        ],
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
        devtool: (isProduction) ? false : "source-map",
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
                },
                {
                    test: /\.html$/i,
                    loader: 'file-loader',
                    options: {
                        name: '[name].[ext]',
                    }
                },
            ]
        }
    }
}
