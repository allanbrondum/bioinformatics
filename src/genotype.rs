use itertools::Itertools;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Genotype {
    Dominant,
    Recessive,
    Heterozygous,
}

impl Genotype {
    pub fn alleles(self) -> &'static [Allele] {
        match self {
            Genotype::Dominant => &[Allele::Dominant],
            Genotype::Recessive => &[Allele::Recessive],
            Genotype::Heterozygous => &[Allele::Dominant, Allele::Recessive],
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct SomaticGene {
    pub genes: Vec<Genotype>,
}

impl FromIterator<Genotype> for SomaticGene {
    fn from_iter<T: IntoIterator<Item = Genotype>>(iter: T) -> Self {
        Self {
            genes: Vec::from_iter(iter),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Allele {
    Dominant,
    Recessive,
}

impl Allele {
    pub fn mix(self, other: Self) -> Genotype {
        match (self, other) {
            (Allele::Dominant, Allele::Dominant) => Genotype::Dominant,
            (Allele::Recessive, Allele::Recessive) => Genotype::Recessive,
            (Allele::Dominant, Allele::Recessive) => Genotype::Heterozygous,
            (Allele::Recessive, Allele::Dominant) => Genotype::Heterozygous,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct GameteGene {
    pub genes: Vec<Allele>,
}

impl FromIterator<Allele> for GameteGene {
    fn from_iter<T: IntoIterator<Item = Allele>>(iter: T) -> Self {
        Self {
            genes: Vec::from_iter(iter),
        }
    }
}

impl GameteGene {
    pub fn mix(&self, other: &Self) -> SomaticGene {
        self.genes
            .iter()
            .zip(other.genes.iter())
            .map(|(gene1, gene2)| gene1.mix(*gene2))
            .collect()
    }
}

pub fn mix_gametes<'a, M1, M2>(gametes1: M1, gametes2: M2) -> Vec<SomaticGene>
where
    M1: IntoIterator<Item = &'a GameteGene>,
    M2: IntoIterator<Item = &'a GameteGene>,
    M2::IntoIter: Clone,
{
    gametes1
        .into_iter()
        .cartesian_product(gametes2)
        .map(|(gamete1, gamete2)| gamete1.mix(gamete2))
        .collect()
}

impl SomaticGene {
    pub fn gametes(&self) -> Vec<GameteGene> {
        let mut res = vec![GameteGene::default()];

        for gene in &self.genes {
            res = res
                .into_iter()
                .cartesian_product(gene.alleles().iter())
                .map(|(mut gene, allele)| {
                    gene.genes.push(*allele);
                    gene
                })
                .collect();
        }

        res
    }
}

#[cfg(test)]
mod test {
    use crate::genotype::Allele::Dominant;
    use crate::genotype::{Allele, GameteGene, Genotype, SomaticGene, mix_gametes};

    #[test]
    fn test_somatic_gene_gametes_1() {
        let gene = SomaticGene::from_iter([Genotype::Dominant]);
        let genes = gene.gametes();
        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].genes[0], Allele::Dominant);

        let gene = SomaticGene::from_iter([Genotype::Recessive]);
        let genes = gene.gametes();
        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].genes[0], Allele::Recessive);

        let gene = SomaticGene::from_iter([Genotype::Heterozygous]);
        let genes = gene.gametes();
        assert_eq!(genes.len(), 2);
        assert_eq!(genes[0].genes[0], Allele::Dominant);
        assert_eq!(genes[1].genes[0], Allele::Recessive);
    }

    #[test]
    fn test_somatic_gene_gametes_2() {
        let gene = SomaticGene::from_iter([Genotype::Dominant, Genotype::Recessive]);
        let genes = gene.gametes();
        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].genes[0], Allele::Dominant);
        assert_eq!(genes[0].genes[1], Allele::Recessive);

        let gene = SomaticGene::from_iter([Genotype::Dominant, Genotype::Heterozygous]);
        let genes = gene.gametes();
        assert_eq!(genes.len(), 2);
        assert_eq!(genes[0].genes[0], Allele::Dominant);
        assert_eq!(genes[0].genes[1], Allele::Dominant);
        assert_eq!(genes[1].genes[0], Allele::Dominant);
        assert_eq!(genes[1].genes[1], Allele::Recessive);

        let gene = SomaticGene::from_iter([Genotype::Heterozygous, Genotype::Heterozygous]);
        let genes = gene.gametes();
        assert_eq!(genes.len(), 4);
        assert_eq!(genes[0].genes[0], Allele::Dominant);
        assert_eq!(genes[0].genes[1], Allele::Dominant);
        assert_eq!(genes[1].genes[0], Allele::Dominant);
        assert_eq!(genes[1].genes[1], Allele::Recessive);
        assert_eq!(genes[2].genes[0], Allele::Recessive);
        assert_eq!(genes[2].genes[1], Allele::Dominant);
        assert_eq!(genes[3].genes[0], Allele::Recessive);
        assert_eq!(genes[3].genes[1], Allele::Recessive);
    }

    #[test]
    fn test_mix_gametes_1() {
        let gametes1 = vec![
            GameteGene::from_iter([Allele::Dominant]),
            GameteGene::from_iter([Allele::Recessive]),
        ];
        let gametes2 = vec![
            GameteGene::from_iter([Allele::Dominant]),
            GameteGene::from_iter([Allele::Recessive]),
        ];
        let genes = mix_gametes(&gametes1, &gametes2);
        assert_eq!(genes.len(), 4);
        assert_eq!(genes[0].genes[0], Genotype::Dominant);
        assert_eq!(genes[1].genes[0], Genotype::Heterozygous);
        assert_eq!(genes[2].genes[0], Genotype::Heterozygous);
        assert_eq!(genes[3].genes[0], Genotype::Recessive);
    }

    #[test]
    fn test_mix_gametes_2() {
        let gametes1 = vec![GameteGene::from_iter([Allele::Dominant, Allele::Recessive])];
        let gametes2 = vec![
            GameteGene::from_iter([Allele::Dominant, Dominant]),
            GameteGene::from_iter([Allele::Recessive, Dominant]),
        ];
        let genes = mix_gametes(&gametes1, &gametes2);
        assert_eq!(genes.len(), 2);
        assert_eq!(genes[0].genes[0], Genotype::Dominant);
        assert_eq!(genes[0].genes[1], Genotype::Heterozygous);
        assert_eq!(genes[1].genes[0], Genotype::Heterozygous);
        assert_eq!(genes[1].genes[0], Genotype::Heterozygous);
    }
}
