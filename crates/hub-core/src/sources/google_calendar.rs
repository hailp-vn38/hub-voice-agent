use crate::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarQuery, CalendarSettings, CalendarSource,
    NewCalendarEvent,
};
use serde::Deserialize;

#[derive(Clone)]
pub struct GoogleCalendar {
    client: reqwest::Client,
    api: String,
    token: String,
}
impl Default for GoogleCalendar {
    fn default() -> Self {
        Self::new(
            "https://www.googleapis.com/calendar/v3",
            "https://oauth2.googleapis.com/token",
        )
    }
}
impl GoogleCalendar {
    pub fn new(api: &str, token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api: api.trim_end_matches('/').into(),
            token: token.into(),
        }
    }
    async fn access(&self, settings: &CalendarSettings) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Token {
            access_token: String,
        }
        Ok(self
            .client
            .post(&self.token)
            .form(&[
                ("client_id", &settings.client_id),
                ("client_secret", &settings.client_secret),
                ("refresh_token", &settings.refresh_token),
                ("grant_type", &"refresh_token".to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Token>()
            .await?
            .access_token)
    }
    async fn list_inner(
        &self,
        settings: CalendarSettings,
        query: CalendarQuery,
    ) -> anyhow::Result<CalendarDelta> {
        let access = self.access(&settings).await?;
        let mut page: Option<String> = None;
        let mut events = Vec::new();
        let mut removed = Vec::new();
        loop {
            let mut request = self
                .client
                .get(format!("{}/calendars/primary/events", self.api))
                .bearer_auth(&access)
                .query(&[("singleEvents", "true"), ("showDeleted", "true")]);
            request = match &query {
                CalendarQuery::Since(token) => request.query(&[("syncToken", token)]),
                CalendarQuery::Full { time_min, time_max } => request.query(&[
                    ("timeMin", time_min.to_string()),
                    ("timeMax", time_max.to_string()),
                ]),
            };
            if let Some(p) = &page {
                request = request.query(&[("pageToken", p)]);
            }
            let response = request.send().await?;
            if response.status() == reqwest::StatusCode::GONE {
                return Ok(CalendarDelta {
                    expired: true,
                    ..Default::default()
                });
            }
            let result: ListResponse = response.error_for_status()?.json().await?;
            for item in result.items {
                if item.status.as_deref() == Some("cancelled") {
                    removed.push(item.id);
                } else if let (Some(start), Some(end)) = (
                    item.start.and_then(Time::value),
                    item.end.and_then(Time::value),
                ) {
                    events.push(CalendarEvent {
                        location: item.location,
                        description: item.description,
                        id: item.id,
                        summary: item.summary.unwrap_or_default(),
                        start,
                        end,
                    });
                }
            }
            if let Some(next) = result.next_page_token {
                page = Some(next);
            } else {
                return Ok(CalendarDelta {
                    events,
                    removed,
                    sync_token: result.next_sync_token,
                    expired: false,
                });
            }
        }
    }
    async fn insert_inner(
        &self,
        settings: CalendarSettings,
        event: NewCalendarEvent,
    ) -> anyhow::Result<CalendarEvent> {
        let access = self.access(&settings).await?;
        let key = if event.start.parse::<jiff::civil::Date>().is_ok() {
            "date"
        } else {
            "dateTime"
        };
        let mut body = serde_json::json!({"summary": event.summary, "start": {key: event.start}, "end": {key: event.end}});
        if let Some(location) = event.location {
            body["location"] = location.into();
        }
        let item: Item = self
            .client
            .post(format!("{}/calendars/primary/events", self.api))
            .bearer_auth(access)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(CalendarEvent {
            location: item.location,
            description: item.description,
            id: item.id,
            summary: item.summary.unwrap_or_default(),
            start: item
                .start
                .and_then(Time::value)
                .ok_or_else(|| anyhow::anyhow!("missing start"))?,
            end: item
                .end
                .and_then(Time::value)
                .ok_or_else(|| anyhow::anyhow!("missing end"))?,
        })
    }
}
impl CalendarSource for GoogleCalendar {
    fn list(
        &self,
        settings: CalendarSettings,
        query: CalendarQuery,
    ) -> CalendarFuture<CalendarDelta> {
        let this = self.clone();
        Box::pin(async move { this.list_inner(settings, query).await })
    }
    fn create(
        &self,
        settings: CalendarSettings,
        event: NewCalendarEvent,
    ) -> CalendarFuture<CalendarEvent> {
        let this = self.clone();
        Box::pin(async move { this.insert_inner(settings, event).await })
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListResponse {
    #[serde(default)]
    items: Vec<Item>,
    next_page_token: Option<String>,
    next_sync_token: Option<String>,
}
#[derive(Deserialize)]
struct Item {
    location: Option<String>,
    description: Option<String>,
    id: String,
    status: Option<String>,
    summary: Option<String>,
    start: Option<Time>,
    end: Option<Time>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Time {
    date_time: Option<String>,
    date: Option<String>,
}
impl Time {
    fn value(self) -> Option<String> {
        self.date_time.or(self.date)
    }
}
