#!/bin/bash
podman build . -t optil
podman rm optilbin
podman create --name optilbin localhost/optil:latest
mkdir optil_target/
rm optil_target/hex
cp extern/WeGotYouCovered/vc_solver optil_target/vc_solver
podman cp optilbin:./target/release/hex optil_target/
tar -czvf optil_target/hex.tgz -C optil_target/ hex vc_solver