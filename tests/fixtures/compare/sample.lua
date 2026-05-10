-- TODO: add error handling
globalVar = 42

function process(items)
    for i, item in ipairs(items) do
        print(item)
    end
    os.execute("ls -la")
    loadstring("return 1")()
end

function empty()
end

function complex(a, b, c, d, e)
    return a + b + c + d + e
end
