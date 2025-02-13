# noti
A command line notification tool.

Supports desktop notifications, and sending messages over webhooks to
Google Chat, Discord, and any supporting plain text using http POST.

## Sending a notification

Below shows how to send a notification once a task has finished.
```sh
# Some long running task.
dbt run --target ... && noti "dbt run complete"
```

This can be useful in cases where your task cannot fail, or where you only want
to be notified that it has finished.

Noti also supports reading from stdin and sending notifications as lines come in.
Naturally this can get quite noisy, so it also features an option to filter input
using regex.

```yaml
# in noti.yaml

stream:
  enabled: true
  matching: "^(WARN:.*)|^(ERROR:.*)"
  redirect: stdout
```

```sh
# Long running task 
dbt run --target ... | noti
```

The above will only send notifications for inputs that start with either `WARN:`
or `ERROR:`.

## Configuration
You can generate sample config files using `noti init desktop` for desktop
notifications, or `noti init webhook` for webhooks.

A `noti.yaml` file has two keys, `destination` and `stream`.

### Destination

The destination key is an array of objects describing where to send your notifications.

Here is an example for configuring a discord webhook, and desktop notifications.

```yaml
destination:
- type: webhook
  url: https://discord.com/api/webhooks/<CHANNEL_ID>/<WEBHOOK_ID>
  format: discord
- type: desktop
  summary: Task finished
  persistent: true
```


| type    | key        | value                                                  | accepted values                                   |
|---------|------------|--------------------------------------------------------|---------------------------------------------------|
| webhook | url        | The url of the webhook to send messages to             | `Any URL`                                         |
| webhook | format     | Which format the webhook requires                      | `discord`, `google_chat`, `plain_text`, `custom`* |
| desktop | summary    | The summary on the notification toast                  | `Any text`                                        |
| desktop | persistent | (true) Notification will stay until manually dismissed | `true` `false`                                    |

When using `custom` webhooks, destinations should be formatted as such:

```yaml
destination:
- type: webhook
  url: https://my.webhook/...
  format:
    content_type: application/json  # Sent in the Content-Type header
    template: '{"content": "$(message)"}'  # $(message) is the placeholder for the message being sent
    escape: true  # Whether to escape special characters in the incoming message
```

### Stream

The stream key is an object that determines whether to listen to stdin for input or not.

Here is an example for listening to error level logs from another program and sending the
message as a notification, while writing the matching logs back out to stderr.

```yaml
stream:
  enabled: true
  matching: "^ERROR:(.*)$"
  redirect: stderr
```
