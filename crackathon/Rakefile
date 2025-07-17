task default: :build

task :build do
  sh 'cargo build'
end

task :test do
  sh 'cargo test'
end

task install: [:test] do
  sh 'cargo build --release'
  sh 'mv ./target/release/crackathon ~/.local/bin/'
end
