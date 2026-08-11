# @prompt 00_nucleo/prompts/core.md
# @layer L1
# @updated 2026-06-08
defmodule Core do
  def read(path) do
    File.read!(path)
  end
end
