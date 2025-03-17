local run = function(cmd)
	local handle = io.popen(cmd)
	if not handle then
		return "failed to cd"
	end
	if type(handle) == "string" then
		return handle
	end
	local output = handle:read("*a")
	handle:close()
	local i = 0
	-- Below taken from <https://www.lua.org/pil/20.1.html>, with some modification
	while true do
		local next = string.find(output, "\n", i + 1) -- find 'next' newline
		if next == nil then
			break
		end
		i = next
	end
	local exit = string.sub(output, i - 1)
	-- Why can't I trim whitespace?
	local code = string.sub(exit, 0, 1)
	if code == "0" then
		return nil
	end
	return code
end
local dir = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")
-- Have to echo $? so that I can parse out the exit code
local cmd = string.format(
	"sh -c 'cd %s && cargo b -r -p neocase &> /dev/null && cp target/release/libneocase.so ./lua/neocase.so; echo $?'",
	dir
)
local run_err = run(cmd)
if run_err then
	error(string.format("build failed", run_err), 3)
end