# Contributing Guide For Heroku Front-end Web CNB

This page lists the operational governance model of this project, as well as the recommendations and requirements for how to best contribute to Heroku Front-end Web CNB. We strive to obey these as best as possible. As always, thanks for contributing – we hope these guidelines make it easier and shed some light on our approach and processes.

# Governance Model

## Salesforce Sponsored

The intent and goal of open sourcing this project is to increase the contributor and user base. However, only Salesforce employees will be given `admin` rights and will be the final arbitrars of what contributions are accepted or not.

# Issues, requests & ideas

This project is in an early experimental phase, and so we are not yet ready to support and accept community contributions.

# Code of Conduct
Please follow our [Code of Conduct](CODE_OF_CONDUCT.md).

# License
By contributing your code, you agree to license your contribution under the terms of our project [LICENSE](LICENSE.txt) and to sign the [Salesforce CLA](https://cla.salesforce.com/sign-cla)

# Adding new framework buildpacks for website-nodjs
To add support for a nodejs website framework, create a new project in the `buildpacks` directory. Follow the `website-ember` buildpack example and make sure to
1. Add an implementation test
1. Update the [buildpack.toml](./meta-buildpacks/website-nodejs/buildpack.toml) in the website-nodejs buildpack to add a new ``[[order]]`` for your buildpack group, including the nodejs and static-web-server buildpacks
1. Update the [package.toml](./meta-buildpacks/website-nodejs/package.toml) in the website-nodejs buildpack to add your buildpack as a dependency
1. Update the [Cargo.toml](./Cargo.toml) file to add your new buildpack to the workspace
1. Update the READMEs in the [root](./README.md) and in [website-nodejs](./meta-buildpacks/website-nodejs/README.md)
