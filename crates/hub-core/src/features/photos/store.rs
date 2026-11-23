use super::Photo;
use crate::{FetchedImage, Store};
use anyhow::{Context, bail};
use rusqlite::{OptionalExtension, params};

impl Store {
    pub fn photos_load(&self) -> anyhow::Result<Vec<Photo>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, caption, created, pinned FROM photos ORDER BY created DESC, rowid DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            let id: String = r.get(0)?;
            Ok(Photo {
                image_path: self
                    .photos_dir
                    .join(format!("{id}.png"))
                    .to_string_lossy()
                    .into_owned(),
                thumbnail_path: self
                    .photos_dir
                    .join(format!("{id}-thumb.png"))
                    .to_string_lossy()
                    .into_owned(),
                id,
                caption: r.get(1)?,
                created: r.get(2)?,
                pinned: r.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
    pub fn photos_pin(&mut self, id: &str, pinned: bool) -> anyhow::Result<()> {
        if self.connection.execute(
            "UPDATE photos SET pinned=?1 WHERE id=?2",
            params![pinned, id],
        )? == 0
        {
            bail!("unknown photo");
        }
        Ok(())
    }
    pub fn photos_add(
        &mut self,
        caption: String,
        image: FetchedImage,
        now: i64,
    ) -> anyhow::Result<()> {
        let dir = &self.photos_dir;
        let size = image.image.len() + image.thumbnail.len();
        if size > 200 * 1024 * 1024 {
            bail!("image exceeds cache limit");
        }
        std::fs::create_dir_all(dir)?;
        let id = uuid::Uuid::new_v4().to_string();
        let main = dir.join(format!("{id}.png"));
        let thumb = dir.join(format!("{id}-thumb.png"));
        std::fs::write(&main, &image.image)?;
        if let Err(e) = std::fs::write(&thumb, &image.thumbnail) {
            let _ = std::fs::remove_file(&main);
            return Err(e.into());
        }
        if let Err(e) = self.connection.execute(
            "INSERT INTO photos (id,caption,created,pinned,size) VALUES (?1,?2,?3,0,?4)",
            params![id, caption, now, size],
        ) {
            let _ = std::fs::remove_file(main);
            let _ = std::fs::remove_file(thumb);
            return Err(e.into());
        }
        loop {
            let (count, bytes): (i64, i64) = self.connection.query_row(
                "SELECT count(*), coalesce(sum(size),0) FROM photos",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            if count <= 100 && bytes <= 200 * 1024 * 1024 {
                break;
            }
            let victim: String = self
                .connection
                .query_row(
                    "SELECT id FROM photos WHERE pinned=0 ORDER BY created, rowid LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?
                .context("photo cache full")?;
            self.connection
                .execute("DELETE FROM photos WHERE id=?1", [&victim])?;
            let _ = std::fs::remove_file(dir.join(format!("{victim}.png")));
            let _ = std::fs::remove_file(dir.join(format!("{victim}-thumb.png")));
            if victim == id {
                bail!("photo cache full");
            }
        }
        Ok(())
    }
}
