# Powershell script to send desktop notification on download completion of a torrent by TCAD.
# Ref: https://steemit.com/powershell/@esoso/fun-with-toast-notifications-in-powershell

param(
    [String] $torrent,
    [String] $filepath
)

[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.UI.Notifications.ToastNotification, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

$app_id = 'TCloud Automatic Downloader'
$notification_template = @"
<toast>
    <visual>
        <binding template="ToastText02">
            <text id="2">TCAD: Download Complete</text>
            <text id="1">$($torrent)</text>
        </binding>  
    </visual>
    <actions>
        <action activationType="protocol" content="Open Directory" arguments="file:///$($filepath)" />
    </actions>
</toast>
"@

$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml($notification_template)
$toast = New-Object Windows.UI.Notifications.ToastNotification $xml
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($app_id).Show($toast)

