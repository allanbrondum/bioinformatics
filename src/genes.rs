use itertools::Itertools;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Genotype {
    Dominant,
    Recessive,
    Heterozygous,
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

impl Genotype {
    pub fn alleles(&self) -> &'static [Allele] {
        match self {
            Genotype::Dominant => &[Allele::Dominant],
            Genotype::Recessive => &[Allele::Recessive],
            Genotype::Heterozygous => &[Allele::Dominant, Allele::Recessive],
        }
    }
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
    use crate::genes::{Allele, Genotype, SomaticGene};

    #[test]
    fn test_gametes_1_gene() {
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
    fn test_gametes_2_genes() {
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
}
