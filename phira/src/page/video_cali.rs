prpr_l10n::tl_file!("cali");

use std::borrow::Cow;

use super::{Page, SharedState};
use crate::{get_data, get_data_mut, save_data};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use prpr::{
    core::{ResourcePack, NOTE_WIDTH_RATIO_BASE},
    ext::{create_audio_manger, semi_black, RectExt, SafeTexture, ScaleType},
    time::TimeManager,
    ui::{Slider, Ui},
};
use sasa::{AudioClip, AudioManager, Music, MusicParams};

pub struct VideoCalibrationPage {
    _audio: AudioManager,
    cali: Music,

    tm: TimeManager,
    flash_state: bool,

    click: SafeTexture,
    color: Color,

    slider: Slider,
}

impl VideoCalibrationPage {
    pub async fn new() -> Result<Self> {
        let mut audio = create_audio_manger(&get_data().config)?;
        let cali = audio.create_music(
            AudioClip::new(load_file("cali.ogg").await?)?,
            MusicParams {
                loop_mix_time: 0.,
                ..Default::default()
            },
        )?;

        let mut tm = TimeManager::new(1., true);
        tm.force = 3e-2;

        let respack = ResourcePack::from_path(get_data().config.res_pack_path.as_ref())
            .await
            .context("Failed to load resource pack")?;
        let click = respack.note_style.click.clone();
        Ok(Self {
            _audio: audio,
            cali,
            tm,
            flash_state: false,

            click,
            color: respack.info.fx_perfect(),

            slider: Slider::new(-500.0..500.0, 5.),
        })
    }
}

impl Page for VideoCalibrationPage {
    fn can_play_bgm(&self) -> bool {
        false
    }

    fn label(&self) -> Cow<'static, str> {
        tl!("label-video")
    }

    fn exit(&mut self) -> Result<()> {
        save_data()?;
        Ok(())
    }

    fn enter(&mut self, _s: &mut SharedState) -> Result<()> {
        self.cali.seek_to(0.)?;
        self.cali.play()?;
        self.tm.reset();
        self.flash_state = false;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        save_data()?;
        self.tm.pause();
        self.cali.pause()?;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.tm.resume();
        self.cali.play()?;
        Ok(())
    }

    fn touch(&mut self, touch: &Touch, s: &mut SharedState) -> Result<bool> {
        let t = s.t;
        let config = &mut get_data_mut().config;
        let mut offset = config.video_offset * 1000.;
        if self.slider.touch(touch, t, &mut offset).is_some() {
            config.video_offset = offset / 1000.;
            return Ok(true);
        }
        Ok(false)
    }

    fn update(&mut self, _s: &mut SharedState) -> Result<()> {
        if !self.cali.paused() {
            let pos = self.cali.position();
            let now = self.tm.now();
            if now > 2. {
                self.tm.seek_to(now - 2.);
                self.tm.dont_wait();
            }
            let now = self.tm.now();
            if now - pos >= -1. {
                self.tm.update(pos);
            }
        }
        Ok(())
    }

    fn render(&mut self, ui: &mut Ui, s: &mut SharedState) -> Result<()> {
        let t = s.t;
        s.render_fader(ui, |ui| {
            let lf = -0.92;
            let mut r = ui.content_rect();
            r.w += r.x - lf;
            r.x = lf;
            ui.fill_path(&r.rounded(0.02), semi_black(0.4));

            let ct = (-0.4, r.bottom() - 0.12);
            let hw = 0.4;
            let hh = 0.005;
            ui.fill_rect(Rect::new(ct.0 - hw, ct.1 - hh, hw * 2., hh * 2.), WHITE);

            let ot = t;

            let config = &get_data().config;
            let mut t = self.tm.now() as f32 - config.video_offset;
            if t < 0. {
                t += 2.;
            }
            if t >= 2. {
                t -= 2.;
            }
            let ny = ct.1 + (t - 1.) * 0.6;
            if t <= 1. {
                let w = NOTE_WIDTH_RATIO_BASE as f32 * config.note_scale * 2.;
                let h = w * self.click.height() / self.click.width();
                let r = Rect::new(ct.0 - w / 2., ny, w, h);
                ui.fill_rect(r, (*self.click, r, ScaleType::Fit));
                self.flash_state = true;
            } else {
                if self.flash_state {
                    let glow_r = Rect::new(ct.0 - hw, ct.1 - hh, hw * 2., hh * 2.);
                    ui.fill_rect(glow_r, self.color);
                }
                self.flash_state = false;
            }

            let offset = config.video_offset * 1000.;
            self.slider
                .render(ui, Rect::new(0.46, -0.1, 0.45, 0.2), ot, offset, format!("{offset:.0}ms"));
        });

        Ok(())
    }
}
