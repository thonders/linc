local submitBtn = gurt.select('#submit')
local url_input = gurt.select('#url')
local slug_input = gurt.select('#slug')
local log_output = gurt.select('#log-output')

function addLog(message)
  trace.log(message)
  log_output.text = log_output.text .. message .. '\n'
end

function clearLog()
  log_output.text = ''
end

function validateForm(url)
  if not url or url == '' then
    addLog('Error: URL is required')
    return false
  end

  if not string.match(url, '^gurt://') then
    addLog('Error: Only gurt:// URLs are supported')
    return false
  end
  
  return true
end

submitBtn:on('submit', function(event)
  local url = event.data.url
  local slug = event.data.slug

  clearLog()
  
  if not validateForm(url) then
    return
  end

  local request_body = JSON.stringify({
    url = url,
    slug = slug
  })
  
  local apiUrl = 'http://linc.thond.re/api/shorten'
  local headers = {
    ['Content-Type'] = 'application/json'
  }

  addLog('Creating short URL for ' .. url)

  local response = fetch(apiUrl, {
    method = 'POST',
    headers = headers,
    body = request_body
  })
  
  addLog('Response Status: ' .. response.status .. ' ' .. response.statusText)
  
  if response:ok() then
    addLog('URL created successfully!')
    local jsonData = response:json()
    if jsonData then
      addLog('Short URL: gurt://li.nc/' .. jsonData.user.short_url)
    end
  else
    addLog('Creation failed with status: ' .. response.status)
    local error_data = response:text()
    addLog('Error: ' .. error_data)
  end
end)
