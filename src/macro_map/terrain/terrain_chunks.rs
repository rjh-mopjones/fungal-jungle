use std::collections::HashMap;
use std::time::SystemTime;
use bevy::prelude::Component;
use crate::macro_map::terrain::noise_layers::{NoiseLayers, NoiseStrategies, NoiseStrategy, NoiseValues as ExternalNoiseValues, NoiseValues};
use crate::macro_map::terrain::tiling::TilingStrategy;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct MesoChunk {
    coord: ChunkCoord,
    size: usize,
    noise_values: Vec<NoiseValues>,
    noise_layers: NoiseLayers,
    last_accessed: SystemTime
}

impl MesoChunk {
    pub fn new(size: usize, coord: ChunkCoord, tiling_strategy: &TilingStrategy, noise_strategies: &NoiseStrategies) -> Self {
        let mut noise_layers = NoiseLayers::default();
        let mut noise_values = vec![NoiseValues::default(); size * size];

        for y in 0..size {
            for x in 0..size {
                let world_x = coord.x as f64 + x as f64 / size as f64;
                let world_y = coord.y as f64 + y as f64 / size as f64;

                let index = y * size + x;
                let current_noise_value = noise_strategies.generate(world_x, world_y, 1);
                noise_layers.add_at_index(x, y, &current_noise_value, tiling_strategy);
                noise_values[index] = current_noise_value;
                noise_strategies.generate(world_x, world_y, 0);
            }
        }

        Self {
            coord,
            size,
            noise_values: vec![NoiseValues::default(); size * size],
            noise_layers: NoiseLayers::default(),
            last_accessed: SystemTime::now(),
        }
    }
}

#[derive(Component)]
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
    pub fn new(size: usize, coord: ChunkCoord, tiling_strategy: &TilingStrategy, noise_strategies: &NoiseStrategies) -> Self {
        let mut noise_layers = NoiseLayers::default();
        let mut noise_values = vec![NoiseValues::default(); size * size];

        for y in 0..size {
            for x in 0..size {
                let world_x = coord.x as f64 + x as f64 / size as f64;
                let world_y = coord.y as f64 + y as f64 / size as f64;

                let index = y * size + x;
                let current_noise_value = noise_strategies.generate(world_x, world_y, 0);
                noise_layers.add_at_index(x, y, &current_noise_value, tiling_strategy);
                noise_values[index] = current_noise_value;
            }
        }

        Self {
            coord,
            size, 
            noise_values: vec![NoiseValues::default(); size * size],
            noise_layers: NoiseLayers::default(),
            last_accessed: SystemTime::now(),
            meso_cache: HashMap::new(),
            max_meso_chunks: 128,
        }
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
        let mut world_x = 0;
        let mut world_y = 0;
        let mut macro_chunks = HashMap::new();

        for y in 0..chunks_y {
            for x in 0..chunks_x {
                world_x = x * chunking_config.macro_chunk_size;
                world_y = y * chunking_config.macro_chunk_size;
                let chunk_coord = ChunkCoord{x: (x * chunking_config.macro_chunk_size) as i32,
                                             y: (y * chunking_config.macro_chunk_size) as i32 };
                macro_chunks.insert(chunk_coord, MacroChunk::new(chunking_config.macro_chunk_size,
                                                                 chunk_coord, &tiling_strategy,
                                                                 &noise_strategies));
            }
        }

        Self {
            macro_chunks,
            chunking_config,
            noise_strategies,
            tiling_strategy
        }
    }

    pub fn get_or_generate_macro(&mut self, coord: ChunkCoord) -> &mut MacroChunk {
        if !self.macro_chunks.contains_key(&coord) {
                self.macro_chunks.insert(coord, MacroChunk::new(32, coord, &self.tiling_strategy, &self.noise_strategies));
        }
        self.macro_chunks.get_mut(&coord).unwrap()
    }
}