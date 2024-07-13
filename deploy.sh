#!/usr/bin/env bash
set -euo pipefail

# Script taken from https://cli.vuejs.org/guide/deployment.html#github-pages

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# build
cd "${DIR}"
#wasm-pack build --target web

rm -rf build
mkdir -p build/examples
cp index.html build/
cp -r pkg/ build/
rm build/pkg/.gitignore
cp 'examples/A Million Stars.png' 'examples/A Million Stars.mid' build/examples/

DEPLOY_URL="git@github.com:FuegoFro/magic_piano.git"

# navigate into the build output directory and push to remote
cd "${DIR}/build"
#ln -s index.html 404.html
#echo -n "<DOMAIN HERE>" > CNAME
git init
git add -A
git commit -m 'deploy'
# Deploy to https://<USERNAME>.github.io/<REPO>
git push -f "${DEPLOY_URL}" HEAD:gh-pages
