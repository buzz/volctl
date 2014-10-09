#!/bin/sh

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source ${DIR}/venv/bin/activate
${DIR}/volctl.py 2>&1 > /dev/null
