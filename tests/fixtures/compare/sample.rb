# Realistic Ruby code with various issues
# TODO: refactor this module

def process(items)
  items.each do |item|
    puts item
    if item > 10
      if item > 20
        puts "big"
      end
    end
  end
end

def empty_method
end

def complex(a, b, c, d, e)
  a + b + c + d + e
end
