# Copyright (C) 2025-2026 Michael S. Klishin and Contributors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Minimal Elixir echo server for testing edp-rs interoperability.
#
# Usage:
#   elixir --sname echo --cookie secret echo_server.exs
#
# The server provides a single function that echoes back any term it receives.

defmodule EchoServer do
  def echo(term) do
    IO.puts("Received: #{inspect(term)}")
    term
  end

  def echo_tagged(term) do
    result = {:echo, term}
    IO.puts("Received: #{inspect(term)} -> returning #{inspect(result)}")
    result
  end
end

IO.puts("Echo server started on node #{Node.self()}")
IO.puts("")
IO.puts("Available functions:")
IO.puts(" * EchoServer.echo(term): returns the term unchanged")
IO.puts(" * EchoServer.echo_tagged(term): returns {:echo, term}")
IO.puts("")
IO.puts("Press Ctrl+C twice to exit")

# Keep the script running
Process.sleep(:infinity)
