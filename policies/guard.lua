-- policies/guard.lua

function check_traffic(payload)
    if not payload then
        return false
    end

    local blocked_keywords = {"confidential", "top secret"}

    for _, keyword in ipairs(blocked_keywords) do
        if string.find(string.lower(payload), keyword) then
            return true -- block
        end
    end

    return false -- pass
end
