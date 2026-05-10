# expect: no-io-inspect, no-io-puts
# expect: todo-comment

# TODO: remove debug
defmodule Violations do
  def debug(x) do
    IO.inspect(x)
    IO.puts("hello")
    x
  end
end
