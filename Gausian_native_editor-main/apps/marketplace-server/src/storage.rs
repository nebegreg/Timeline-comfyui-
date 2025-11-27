/// Simple JSON-based storage for marketplace data
/// In production, this would use a proper database
use crate::models::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MarketplaceStorage {
    storage_path: PathBuf,
    plugins: RwLock<HashMap<String, Plugin>>,
    ratings: RwLock<HashMap<String, Vec<Rating>>>,
}

impl MarketplaceStorage {
    pub fn new(storage_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();
        fs::create_dir_all(&storage_path)?;

        let plugins_file = storage_path.join("plugins.json");
        let ratings_file = storage_path.join("ratings.json");

        let plugins = if plugins_file.exists() {
            let data = fs::read_to_string(&plugins_file)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let ratings = if ratings_file.exists() {
            let data = fs::read_to_string(&ratings_file)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            storage_path,
            plugins: RwLock::new(plugins),
            ratings: RwLock::new(ratings),
        })
    }

    pub fn list_plugins(&self, query: &SearchQuery) -> SearchResults {
        let plugins_lock = self.plugins.read();
        let mut plugins: Vec<Plugin> = plugins_lock.values().cloned().collect();

        // Filter by category
        if let Some(cat) = &query.category {
            plugins.retain(|p| p.category == *cat);
        }

        // Filter by type
        if let Some(ptype) = &query.plugin_type {
            plugins.retain(|p| p.plugin_type == *ptype);
        }

        // Filter by tag
        if let Some(tag) = &query.tag {
            plugins.retain(|p| p.tags.contains(tag));
        }

        // Search
        if let Some(search) = &query.q {
            let search_lower = search.to_lowercase();
            plugins.retain(|p| {
                p.name.to_lowercase().contains(&search_lower)
                    || p.description.to_lowercase().contains(&search_lower)
                    || p.tags.to_lowercase().contains(&search_lower)
            });
        }

        // Sort
        match query.sort.as_deref() {
            Some("downloads") => plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads)),
            Some("rating") => plugins.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap()),
            Some("recent") => plugins.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            _ => plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads)),
        }

        let total = plugins.len() as i64;
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = ((page - 1) * limit) as usize;

        let plugins = plugins
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();

        SearchResults {
            plugins,
            total,
            page,
            pages: (total + limit - 1) / limit,
        }
    }

    pub fn get_plugin(&self, id: &str) -> Option<Plugin> {
        self.plugins.read().get(id).cloned()
    }

    pub fn add_plugin(&self, plugin: Plugin) -> anyhow::Result<Plugin> {
        let mut plugins = self.plugins.write();
        plugins.insert(plugin.id.clone(), plugin.clone());
        drop(plugins);

        self.save_plugins()?;
        Ok(plugin)
    }

    pub fn increment_downloads(&self, id: &str) -> anyhow::Result<()> {
        let mut plugins = self.plugins.write();
        if let Some(plugin) = plugins.get_mut(id) {
            plugin.downloads += 1;
        }
        drop(plugins);

        self.save_plugins()
    }

    pub fn add_rating(&self, plugin_id: String, rating: Rating) -> anyhow::Result<Rating> {
        let mut ratings = self.ratings.write();
        ratings
            .entry(plugin_id.clone())
            .or_insert_with(Vec::new)
            .push(rating.clone());
        drop(ratings);

        // Update plugin rating
        self.update_plugin_rating(&plugin_id)?;
        self.save_ratings()?;

        Ok(rating)
    }

    pub fn get_ratings(&self, plugin_id: &str) -> Vec<Rating> {
        self.ratings
            .read()
            .get(plugin_id)
            .cloned()
            .unwrap_or_default()
    }

    fn update_plugin_rating(&self, plugin_id: &str) -> anyhow::Result<()> {
        let ratings_lock = self.ratings.read();
        if let Some(plugin_ratings) = ratings_lock.get(plugin_id) {
            if !plugin_ratings.is_empty() {
                let avg_rating: f32 = plugin_ratings.iter().map(|r| r.rating as f32).sum::<f32>()
                    / plugin_ratings.len() as f32;
                let rating_count = plugin_ratings.len() as i32;

                drop(ratings_lock);

                let mut plugins = self.plugins.write();
                if let Some(plugin) = plugins.get_mut(plugin_id) {
                    plugin.rating = avg_rating;
                    plugin.rating_count = rating_count;
                }
                drop(plugins);

                self.save_plugins()?;
            }
        }

        Ok(())
    }

    fn save_plugins(&self) -> anyhow::Result<()> {
        let plugins = self.plugins.read();
        let json = serde_json::to_string_pretty(&*plugins)?;
        fs::write(self.storage_path.join("plugins.json"), json)?;
        Ok(())
    }

    fn save_ratings(&self) -> anyhow::Result<()> {
        let ratings = self.ratings.read();
        let json = serde_json::to_string_pretty(&*ratings)?;
        fs::write(self.storage_path.join("ratings.json"), json)?;
        Ok(())
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let plugins = self.plugins.read();
        let total_plugins = plugins.len();
        let total_downloads: i64 = plugins.values().map(|p| p.downloads).sum();
        let verified_plugins = plugins.values().filter(|p| p.verified).count();

        serde_json::json!({
            "total_plugins": total_plugins,
            "total_downloads": total_downloads,
            "verified_plugins": verified_plugins,
        })
    }
}
