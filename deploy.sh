#!/usr/bin/env bash
set -euo pipefail

# Script taken from https://cli.vuejs.org/guide/deployment.html#github-pages

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# build
cd "${DIR}"
# The --no-sri is because I was getting a weird error otherwise.
# TODO - report bug and re-enable SRI
trunk build --public-url "https://fuegofro.github.io/magic_piano/" --no-sri

DEPLOY_URL="git@github.com:FuegoFro/magic_piano.git"

# navigate into the build output directory and push to remote
cd "${DIR}/dist"
#ln -s index.html 404.html
#echo -n "<DOMAIN HERE>" > CNAME
git init
git add -A
git commit -m 'deploy'
# Deploy to https://<USERNAME>.github.io/<REPO>
git push -f "${DEPLOY_URL}" HEAD:gh-pages
