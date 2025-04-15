sh cargo_login.sh

rm -rf ~/pgwire-lite-rs
mkdir -p ~/pgwire-lite-rs
cp -r /mnt/c/LocalGitRepos/stackql/clients/pgwire-lite-rs/* ~/pgwire-lite-rs/
cargo publish --dry-run
cargo package --list
cargo publish

cd /mnt/c/LocalGitRepos/stackql/clients/pgwire-lite-rs