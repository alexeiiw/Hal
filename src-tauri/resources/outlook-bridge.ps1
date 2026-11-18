param([switch]$Once, [int]$ParentPid = 0)

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = 'Stop'
$clsid = '{0006F03A-0000-0000-C000-000000000046}'
$culture = [System.Globalization.CultureInfo]::CurrentCulture

function Send-Payload($payload) {
    $payload | ConvertTo-Json -Compress -Depth 5
    [Console]::Out.Flush()
}

while ($true) {
    if ($ParentPid -gt 0 -and -not (Get-Process -Id $ParentPid -ErrorAction SilentlyContinue)) {
        break
    }

    $outlook = Get-Process outlook -ErrorAction SilentlyContinue
    if (-not $outlook) {
        Send-Payload @{ running = $false; ok = $false; error = 'OUTLOOK CERRADO'; unread = 0; events = @(); mail = @() }
        if ($Once) { break }
        Start-Sleep -Seconds 20
        continue
    }

    $app = $null
    try {
        $app = [Activator]::CreateInstance([type]::GetTypeFromCLSID($clsid))
        $session = $app.GetNamespace('MAPI')

        $inbox = $session.GetDefaultFolder(6)
        $unread = [int]$inbox.UnReadItemCount

        $mail = New-Object System.Collections.ArrayList
        $items = $inbox.Items
        $items.Sort('[ReceivedTime]', $true)
        $item = $items.GetFirst()
        $count = 0
        while ($item -and $count -lt 5) {
            try {
                [void]$mail.Add(@{
                    id       = [string]$item.EntryID
                    sender   = [string]$item.SenderName
                    subject  = [string]$item.Subject
                    received = ([DateTimeOffset]$item.ReceivedTime).ToUnixTimeSeconds()
                    unread   = [bool]$item.UnRead
                })
            } catch { }
            $count++
            $item = $items.GetNext()
        }

        $calendar = $session.GetDefaultFolder(9)
        $appointments = $calendar.Items
        $appointments.Sort('[Start]')
        $appointments.IncludeRecurrences = $true
        $from = (Get-Date).Date
        $to = $from.AddDays(2)
        $filter = "[Start] >= '" + $from.ToString('dd/MM/yyyy HH:mm', $culture) + "' AND [Start] < '" + $to.ToString('dd/MM/yyyy HH:mm', $culture) + "'"
        $range = $appointments.Restrict($filter)

        $events = New-Object System.Collections.ArrayList
        $appointment = $range.GetFirst()
        $count = 0
        while ($appointment -and $count -lt 12) {
            try {
                [void]$events.Add(@{
                    id      = [string]$appointment.GlobalAppointmentID
                    subject = [string]$appointment.Subject
                    start   = ([DateTimeOffset]$appointment.Start).ToUnixTimeSeconds()
                    end     = ([DateTimeOffset]$appointment.End).ToUnixTimeSeconds()
                    allDay  = [bool]$appointment.AllDayEvent
                })
            } catch { }
            $count++
            $appointment = $range.GetNext()
        }

        Send-Payload @{ running = $true; ok = $true; error = $null; unread = $unread; events = @($events); mail = @($mail) }
    } catch {
        Send-Payload @{ running = $true; ok = $false; error = [string]$_.Exception.Message; unread = 0; events = @(); mail = @() }
    } finally {
        if ($app) {
            try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($app) } catch { }
        }
    }

    if ($Once) { break }
    Start-Sleep -Seconds 45
}
