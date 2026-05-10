# Realistic Elixir code with various issues
defmodule Sample do
  # TODO: remove debug calls

  def process(data) do
    IO.inspect(data, label: "debug")
    IO.puts("processing...")

    result = data
    |> Enum.map(&(&1 * 2))
    |> Enum.filter(&(&1 > 5))

    result
  end

  def empty_func do
  end

  def risky(x) do
    with {:ok, val} <- fetch(x) do
      raise "unexpected"
    end
  end

  defp fetch(_x), do: {:ok, 42}
end
