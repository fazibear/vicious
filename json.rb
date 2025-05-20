#!/usr/bin/env ruby

require 'json'
require 'time'
require 'set'
require 'pathname'

CURRENT_DIR = Pathname.new('.')
root = ARGV[0] || 'C64Music'

unless Dir.exist?(root)
  warn "Error: Directory not found: #{root}"
  exit 1
end

def entry_hash(path)
  stat = File.lstat(path)
  entry = {
    type: stat.directory? ? 'directory' : 'file',
    name: File.basename(path),
    path: Pathname.new(path).relative_path_from(CURRENT_DIR).to_s,
  }
  if stat.directory?
    real = File.realpath(path) rescue nil
    if real
      children = Dir.entries(path).reject { |e| e == '.' || e == '..' }.sort.map do |child|
        entry_hash(File.join(path, child))
      end
      entry[:children] = children
    else
      entry[:children] = []
    end
  end

  entry
end

tree = entry_hash(root)

File.write('C64Music.json', tree.to_json)
