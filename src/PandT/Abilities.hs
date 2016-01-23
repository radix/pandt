-- | A bunch of abilities. This module should probably go in a GameDesign sub-module or something.

module PandT.Abilities where

import ClassyPrelude
import PandT.Types
import PandT.Sim

punchTEffect :: TargetedEffect
punchTEffect = SingleTargetedEffect $ TargetedEffectP "Stab" (TargetCreature (Range 1)) punchEffect

punchEffect :: Effect
punchEffect = Damage 3

punch :: Ability
punch = Ability "Punch" (Energy 10) [punchTEffect] (CastTime 0) (Cooldown 0)

sootheTEffect :: TargetedEffect
sootheTEffect = SingleTargetedEffect $ TargetedEffectP "Soothe" (TargetCreature (Range 5)) (Heal 1)

soothe :: Ability
soothe = Ability "Soothe" (Mana 10) [sootheTEffect] (CastTime 0) (Cooldown 0)

makeTimedEOT :: Text -> Int -> Effect -> Effect
makeTimedEOT cname cdur ceff
    = ApplyCondition (
        MkConditionDef
            cname
            (TimedCondition (Duration cdur))
            (MkRecurringEffectC ceff))

bleed :: Effect
bleed = makeTimedEOT "Bleeding" 2 (Damage 2)

stab :: Ability
stab = Ability
    { _abilityName="Stab"
    , _cost=Energy 10
    , _abilityEffects=[stabTargetedEffect]
    , _castTime = CastTime 0
    , _cooldown = Cooldown 0
    }
    where
        stabTargetedEffect =
            SingleTargetedEffect $ TargetedEffectP
                { _targetedEffectName = "Stab"
                , _targetedEffectSystem = TargetCreature (Range 1)
                , _targetedEffectEffect = stabEffect
                }
        stabEffect = MultiEffect stabDirectDamage bleed
        stabDirectDamage = Damage 3

wrath :: Ability
wrath = Ability "Wrath" (Mana 10) [wrathTEffect] (CastTime 1) (Cooldown 0)
    where
        wrathTEffect = SingleTargetedEffect $ TargetedEffectP "Wrath" (TargetCreature (Range 1)) wrathEffect
        wrathEffect = Damage 5

kill :: Ability
kill = Ability "Kill" (Energy 10) [killTargetedEffect] (CastTime 0) (Cooldown 0)
    where
        killTargetedEffect = SingleTargetedEffect $ TargetedEffectP "Stab" (TargetCreature (Range 1)) killEffect
        killEffect = ApplyCondition deadDef

mkStun :: Text -> Duration -> Effect
mkStun name dur = ApplyCondition (MkConditionDef name (TimedCondition dur) MkIncapacitatedC)

bonk :: Ability
bonk = Ability "Bonk" (Energy 10) [bonkTEffect] (CastTime 0) (Cooldown 0)
    where
        bonkTEffect = SingleTargetedEffect $ TargetedEffectP "Bonk" (TargetCreature (Range 1)) bonkEffect
        bonkEffect = mkStun "Bonked" (Duration 1)
