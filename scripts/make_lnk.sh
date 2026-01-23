#!/bin/bash
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
original_dir="$(pwd)"
cd "$(dirname "$0")" || exit 1
deploy_sh="$script_dir/dev/deploy.sh"
dev_deploy_sh="$script_dir/dev/dev_deploy.sh"
parent_dir="$(dirname "$script_dir")"

if [ -f "$deploy_sh" ]; then
    link_path="$parent_dir/deploy"
    if [ -e "$link_path" ]; then
        rm -f "$link_path"
    fi
    ln -s "$deploy_sh" "$link_path"
fi

if [ -f "$dev_deploy_sh" ]; then
    link_path="$parent_dir/dev"
    if [ -e "$link_path" ]; then
        rm -f "$link_path"
    fi
    ln -s "$dev_deploy_sh" "$link_path"
fi

cd "$original_dir" || exit 1
