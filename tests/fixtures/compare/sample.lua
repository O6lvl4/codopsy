-- Realistic Lua code with various issues
-- TODO: add error handling

function process(items)
    for i, item in ipairs(items) do
        print(item)
        if item > 10 then
            if item > 20 then
                print("big: " .. item)
            end
        end
    end
end

function empty()
end

function complex(a, b, c, d, e)
    return a + b + c + d + e
end
