use std::collections::HashMap;
use std::time::SystemTime;
use crate::macro_map::terrain::noise_layers::{NoiseLayers, NoiseStrategies, NoiseStrategy, NoiseValues};
use crate::macro_map::terrain::tiling::TilingStrategy;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

pub struct MesoChunk {
    coord: ChunkCoord,
    size: usize,
    noise_values: Vec<NoiseValues>,
    noise_layers: NoiseLayers,
    last_accessed: SystemTime
}

impl MesoChunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            size: 64, // Medium detail
            noise_values: vec![NoiseValues::default(); 32 * 32],
            noise_layers: NoiseLayers::default(),
            last_accessed: SystemTime::now(),
        }
    }

    pub fn generate(
        &mut self,
        base_x: f64,
        base_y: f64,
        noise_strategies: NoiseStrategies,
    ) {
        for y in 0..self.size {
            for x in 0..self.size {
                let world_x = base_x + x as f64 / self.size as f64;
                let world_y = base_y + y as f64 / self.size as f64;

                let index = y * self.size + x;
                self.noise_values[index] = noise_strategies.generate(world_x, world_y, 0);
            }
        }
    }

    pub fn sample<'a>(&self, local_x: usize, local_y: usize) -> &'a NoiseValues {
        let idx = local_y * self.size + local_x;
        &self.noise_values[idx]
    }

    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
    }
}

pub struct MacroChunk {
    coord: ChunkCoord,
    size: usize,
    last_accessed: SystemTime,
    meso_cache: HashMap<ChunkCoord, MesoChunk>,
    max_meso_chunks: usize,
    noise_values: Vec<NoiseValues>,
    noise_layers: NoiseLayers,
}

impl MacroChunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            size: 32, // Low detail
            noise_values: vec![NoiseValues::default(); 32 * 32],
            noise_layers: NoiseLayers::default(),
            last_accessed: SystemTime::now(),
            meso_cache: HashMap::new(),
            max_meso_chunks: 128,
        }
    }

    pub fn generate(
        &mut self,
        noise_strategies: NoiseStrategies,
    ) {
        for y in 0..self.size {
            for x in 0..self.size {
                let world_x = self.coord.x as f64 + x as f64 / self.size as f64;
                let world_y = self.coord.y as f64 + y as f64 / self.size as f64;

                let index = y * self.size + x;
                self.noise_values[index] = noise_strategies.generate(world_x, world_y, 0);
            }
        }
    }

    pub fn get_or_generate_meso(
        &mut self,
        meso_coord: ChunkCoord,
        noise_strategies: NoiseStrategies
    ) -> &mut MesoChunk {
        if !self.meso_cache.contains_key(&meso_coord) {
            if self.meso_cache.len() >= self.max_meso_chunks {
                let oldest = self.meso_cache
                    .iter()
                    .min_by_key(|(_, chunk)| chunk.last_accessed)
                    .map(|(coord, _)| *coord);
                if let Some(coord) = oldest {
                    self.meso_cache.remove(&coord);
                }
            }

            let base_x = self.coord.x as f64 + meso_coord.x as f64 / 4.0;
            let base_y = self.coord.y as f64 + meso_coord.y as f64 / 4.0;

            let mut meso = MesoChunk::new(meso_coord);
            meso.generate(base_x, base_y, noise_strategies);
            self.meso_cache.insert(meso_coord, meso);
        }

        let chunk = self.meso_cache.get_mut(&meso_coord).unwrap();
        chunk.touch();
        self.touch();
        chunk
    }

    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
    }
}

pub struct ChunkingConfig {
    pub macro_chunk_size: usize,
    pub meso_chunk_size: usize,
    pub map_width: usize,
    pub map_height: usize
}

pub struct WorldChunks {
    macro_chunks: HashMap<ChunkCoord, MacroChunk>,
    noise_strategies: NoiseStrategies,
    tiling_strategy: TilingStrategy,
    chunking_config: ChunkingConfig
}

impl WorldChunks {
    pub fn new(
               noise_strategies: NoiseStrategies,
               tiling_strategy: TilingStrategy,
               chunking_config: ChunkingConfig) -> Self {
        let chunks_y = chunking_config.map_height / chunking_config.macro_chunk_size;
        let chunks_x = chunking_config.map_width / chunking_config.macro_chunk_size;
        let world_x = 0;
        let mut world_y = 0;

        for y in 0..chunks_y {
            for x in 0..chunks_x {
                let world_x = self.coord.x as f64 + x as f64 / self.size as f64;
                let world_y = self.coord.y as f64 + y as f64 / self.size as f64;

                let index = y * self.size + x;
                self.noise_values[index] = noise_strategies.generate(world_x, world_y, 0);
            }
            world_y = world_y + chunking_config.macro_chunk_size
        }

        Self {
            macro_chunks: HashMap::new(),
            chunking_config,
            noise_strategies,
            tiling_strategy
        }
    }

    pub fn get_or_generate_macro(&mut self, coord: ChunkCoord) -> &mut MacroChunk {
        if !self.macro_chunks.contains_key(&coord) {
            if self.macro_chunks.len() >= self.max_macro_chunks {
                let oldest = self.macro_chunks
                    .iter()
                    .min_by_key(|(_, chunk)| chunk.last_accessed)
                    .map(|(coord, _)| *coord);
                if let Some(coord) = oldest {
                    self.macro_chunks.remove(&coord);
                }
            }

            let mut macro_chunk = MacroChunk::new(coord);
            macro_chunk.generate(&*self.noise_strategies);
            self.macro_chunks.insert(coord, macro_chunk);
        }

        self.macro_chunks.get_mut(&coord).unwrap()
    }

    pub fn sample<'a>(&mut self, x: f64, y: f64, detail_level: u32) -> &'a NoiseValues {
        let macro_x = x.floor() as i32;
        let macro_y = y.floor() as i32;
        let macro_coord = ChunkCoord { x: macro_x, y: macro_y };
        let macro_chunk = self.get_or_generate_macro(macro_coord);

        match detail_level {
            0 => {
                let local_x = ((x - macro_x as f64) * 32.0) as usize;
                let local_y = ((y - macro_y as f64) * 32.0) as usize;
                let idx = local_y * 32 + local_x;
                &macro_chunk.noise_values[idx]
            },
            1 => {
                let meso_x = ((x - macro_x as f64) * 4.0).floor() as i32;
                let meso_y = ((y - macro_y as f64) * 4.0).floor() as i32;
                let meso_coord = ChunkCoord { x: meso_x, y: meso_y };

                let meso_chunk = macro_chunk.get_or_generate_meso(
                    meso_coord,
                    &*self.noise_strategies
                );

                let local_x = ((x - (macro_x as f64 + meso_x as f64 / 4.0)) * 64.0) as usize;
                let local_y = ((y - (macro_y as f64 + meso_y as f64 / 4.0)) * 64.0) as usize;
                meso_chunk.sample(local_x, local_y)
            },
            _ => panic!("Invalid detail level"),
        }
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let meso_count: usize = self.macro_chunks
            .values()
            .map(|chunk| chunk.meso_cache.len())
            .sum();
        (self.macro_chunks.len(), meso_count)
    }
}